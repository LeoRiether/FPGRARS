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

use self::lexer::Lexer;
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
    pub data: Vec<u8>,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelUseType {
    Code,
    Data,
}

#[derive(Debug, Default)]
pub struct ParserContext {
    pub code: Vec<Instruction>,
    pub data: Vec<u8>,
    pub data_type: data::Type,
    pub segment: Segment,
    pub labels: HashMap<Label, usize>,
    /// This parser only makes one pass over the tokens. This means that some instructions will
    /// reference labels which have not yet been defined. When this happens, we store the position
    /// of the instruction or data in the backlog, so when the label is defined we can go back and
    /// fill the labels in.
    pub backlog: HashMap<Label, Vec<LabelUse>>,
    pub regnames: RegNames,
}

impl ParserContext {
    pub fn use_label(&mut self, label: &str, use_type: LabelUseType, ctx: token::Context) -> u32 {
        match self.labels.get(label) {
            Some(&pos) => pos as u32,
            None => {
                let entry = match use_type {
                    LabelUseType::Code => LabelUse::Code(self.code.len(), ctx),
                    LabelUseType::Data => LabelUse::Data(self.data.len(), self.data_type, ctx),
                };
                self.backlog
                    .entry(label.to_string())
                    .or_default()
                    .push(entry);
                0 // dummy value that will be replaced when the label is defined
            }
        }
    }

    pub fn define_label(&mut self, label: impl Into<Label>, value: usize) {
        let label = label.into();
        let backlog = self.backlog.remove(&label);
        self.labels.insert(label, value);

        for use_ in backlog.unwrap_or_default() {
            match use_ {
                LabelUse::Code(i, _) => text::unlabel(&mut self.code, i, value),
                LabelUse::Data(i, t, _) => data::unlabel(&mut self.data, i, t, value as u32),
            }
        }
    }
}

/// Parses a RISC-V file into a `code` and `data` segments.
/// The `data_segment_size` parameter is the final size of the data segment, in bytes.
/// ```
/// parser::parse("riscv.s", DATA_SIZE)?
/// ```
pub fn parse(entry_file: &str, data_segment_size: usize) -> ParseResult {
    let mut tokens = Lexer::new(entry_file).preprocess();
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
                Label(label) => ctx.define_label(label, ctx.data.len()),
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
                })
            })
            .collect();
        ctxs.sort();
        return Err(ParserError::UndefinedLabels(ctxs).into());
    }

    // If the program ever drops off bottom, we make an "exit" ecall and terminate execution
    ctx.code.extend(vec![
        Instruction::Li(17, 10), // li a7 10
        Instruction::Ecall,
    ]);

    ctx.data.resize(data_segment_size, 0);

    Ok(Parsed {
        code: ctx.code,
        data: ctx.data,
    })
}
