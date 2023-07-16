//!
//! Parses RISC-V code into code and data parts, so it can be used in the simulator module.
//! We use a lot of mnemonics here, I'll try to link to a cheatsheet here later.
//!

// TODO: replace unwraps and panics by proper error handling

mod data;
pub mod error;
pub mod lexer;
mod preprocessor;
pub mod register_names;
mod text;
pub mod token;

use std::{iter::Peekable, mem};

use self::{lexer::Lexer, token::Token};
use crate::{
    instruction::{FloatInstruction, Instruction},
    parser::{error::Contextualize, register_names::RegNames},
};
use error::{Error, ParserError};
use hashbrown::HashMap;
pub use preprocessor::Preprocess;

/// Represents a successful parser result. This is the same format the simulator
/// will use to execute the instructions
pub struct Parsed {
    pub code: Vec<Instruction>,
    pub code_ctx: Vec<token::Context>,
    pub data: Vec<u8>,
    pub globl: Option<usize>,
}

pub type ParseResult = Result<Parsed, Error>;

/// The "current" parser directive
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Segment {
    #[default]
    Text,
    Data,
}

type Label = String;

#[derive(Debug, Clone, PartialEq)]
pub enum LabelUse {
    Code(usize, token::Context),
    Data(usize, data::Type, token::Context),
    Globl(token::Context),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelUseType {
    Code,
    Data,
    Globl,
}

#[derive(Debug, Default)]
pub struct ParserContext {
    /// Text segment
    pub code: Vec<Instruction>,
    /// Context for each instruction, for debugging purposes
    pub code_ctx: Vec<token::Context>,
    /// Data segment
    pub data: Vec<u8>,
    /// Current data::Type, like .word, .byte, ...
    pub data_type: data::Type,
    /// Labels that have been defined in .data, but we don't yet know the address of because of
    /// alignment
    pub data_label_backlog: Vec<Label>,
    pub segment: Segment,
    pub labels: HashMap<Label, usize>,
    /// This parser only makes one pass over the tokens. This means that some instructions will
    /// reference labels which have not yet been defined. When this happens, we store the position
    /// of the instruction or data in the backlog, so when the label is defined we can go back and
    /// fill the labels in.
    pub backlog: HashMap<Label, Vec<LabelUse>>,
    pub regnames: RegNames,
    pub globl: Option<usize>,
}

impl ParserContext {
    /// Returns the address of `label`. If the label hasn't been defined yet, we return zero and
    /// add it to the backlog.
    pub fn use_label(&mut self, label: &str, use_type: LabelUseType, ctx: token::Context) -> u32 {
        match self.labels.get(label) {
            Some(&pos) => pos as u32,
            None => {
                let entry = match use_type {
                    LabelUseType::Code => LabelUse::Code(self.code.len(), ctx),
                    LabelUseType::Data => LabelUse::Data(self.data.len(), self.data_type, ctx),
                    LabelUseType::Globl => LabelUse::Globl(ctx),
                };
                self.backlog
                    .entry(label.to_string())
                    .or_default()
                    .push(entry);
                0 // dummy value that will be replaced when the label is defined
            }
        }
    }

    /// When a label is defined, we should call this function to clear the backlog entries related
    /// to it.
    pub fn define_label(&mut self, label: impl Into<Label>, value: usize) {
        let label = label.into();
        let backlog = self.backlog.remove(&label);
        self.labels.insert(label, value);

        for use_ in backlog.unwrap_or_default() {
            match use_ {
                LabelUse::Code(i, _) => text::unlabel(&mut self.code, i, value),
                LabelUse::Data(i, t, _) => data::unlabel(&mut self.data, i, t, value as u32),
                LabelUse::Globl(_) => {
                    self.globl = Some(value);
                }
            }
        }
    }

    /// Defines the address of labels in the data segment with the address self.data.len(). This
    /// should be called after we are sure of the labels alignment
    pub fn commit_data_label_backlog(&mut self) {
        let addr = self.data.len();
        let backlog = mem::take(&mut self.data_label_backlog);
        for label in backlog {
            self.define_label(label, addr);
        }
    }
}

/// Parses a RISC-V file into a `code` and `data` segments.
/// The `data_segment_size` parameter is the final size of the data segment, in bytes.
/// ```
/// parser::parse("riscv.s", DATA_SIZE)?
/// ```
pub fn parse(entry_file: &str, data_segment_size: usize) -> ParseResult {
    let tokens = Lexer::new(entry_file)?.preprocess().peekable();
    parse_tokens(tokens, data_segment_size)
}

pub fn parse_tokens<I: Iterator<Item = Result<Token, Error>>>(
    mut tokens: Peekable<I>,
    data_segment_size: usize,
) -> ParseResult {
    let mut ctx = ParserContext::default();

    use token::Data::*;
    while let Some(token) = tokens.next() {
        let token = token?;

        match token.data {
            Directive(d) if d == "text" => {
                ctx.segment = Segment::Text;
                continue;
            }
            Directive(d) if d == "data" => {
                ctx.segment = Segment::Data;
                continue;
            }
            Directive(d) if d == "globl" || d == "global" => {
                parse_globl(&mut tokens, &mut ctx, token.ctx)?;
                continue;
            }
            _ => {}
        }

        match ctx.segment {
            Segment::Text => match token.data {
                Label(label) => ctx.define_label(label, 4 * ctx.code.len()),
                Identifier(id) => text::parse_instruction(&mut tokens, &mut ctx, id, token.ctx)?,
                Directive(d) => {
                    return Err(ParserError::UnknownDirective(d).with_context(token.ctx))
                }
                _ => {
                    return Err(
                        ParserError::UnexpectedToken(Some(token.data)).with_context(token.ctx)
                    )
                }
            },
            Segment::Data => match token.data {
                Label(label) => ctx.data_label_backlog.push(label),
                Directive(d) if d.parse::<data::Type>().is_ok() => {
                    ctx.data_type = d.parse().unwrap();
                }
                Identifier(_) | CharLiteral(_) | StringLiteral(_) | Integer(_) | Float(_) => {
                    data::push_data(token, &mut ctx)?
                }

                Directive(d) => {
                    return Err(ParserError::UnknownDirective(d).with_context(token.ctx))
                }
                _ => {
                    return Err(
                        ParserError::UnexpectedToken(Some(token.data)).with_context(token.ctx)
                    )
                }
            },
        }
    }

    // Commit labels that were defined without any data, in the end of the backlog, to the position
    // of the end of the data segment
    ctx.commit_data_label_backlog();

    // Check for undefined labels used
    if !ctx.backlog.is_empty() {
        let mut ctxs: Vec<token::Context> = ctx
            .backlog
            .iter()
            .flat_map(|(_label, uses)| {
                // extract contexts
                uses.iter().map(|u| match u {
                    LabelUse::Code(_, c) => c.clone(),
                    LabelUse::Data(_, _, c) => c.clone(),
                    LabelUse::Globl(c) => c.clone(),
                })
            })
            .collect();
        ctxs.sort();
        return Err(ParserError::UndefinedLabels(ctxs).into());
    }

    // If the program ever drops off bottom, we make an "exit" ecall and terminate execution
    ctx.code.extend(vec![
        Instruction::Li(17, 10), // li a7 10
        Instruction::Li(10, 0),  // li a0 0
        Instruction::Ecall,
    ]);

    ctx.data.resize(data_segment_size, 0);

    Ok(Parsed {
        code: ctx.code,
        code_ctx: ctx.code_ctx,
        data: ctx.data,
        globl: ctx.globl,
    })
}

/// Parser a .globl directive
fn parse_globl(
    tokens: &mut Peekable<impl Iterator<Item = Result<token::Token, Error>>>,
    parser: &mut ParserContext,
    globl_ctx: token::Context,
) -> Result<(), Error> {
    let label = tokens
        .next()
        .ok_or_else(|| ParserError::UnexpectedToken(None).with_context(globl_ctx.clone()))??;

    match label.data {
        token::Data::Identifier(label) => {
            let addr = parser.use_label(&label, LabelUseType::Globl, globl_ctx) as usize;
            parser.globl = Some(addr);
            Ok(())
        }
        _ => Err(ParserError::UnexpectedToken(Some(label.data)).with_context(label.ctx)),
    }
}
