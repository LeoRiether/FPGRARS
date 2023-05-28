use crate::parser::error::Contextualize;

use super::error::ParserError;
use super::token::Token;
use super::ParserContext;

use byteorder::{ByteOrder, LittleEndian};
use std::str::FromStr;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    #[default]
    Word,
    Byte,
    Half,
    Align,
    Asciz,
    Float,
}

impl FromStr for Type {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Type::*;
        match s {
            "word" => Ok(Word),
            "byte" => Ok(Byte),
            "half" => Ok(Half),
            "align" | "space" => Ok(Align),
            "asciz" | "ascii" | "string" => Ok(Asciz),
            "float" => Ok(Float),
            _ => Err(ParserError::UnrecognizedDataType(s.to_owned())),
        }
    }
}

/// Stores the information of a label we found in the `.data` directive, so we can
/// later populate the memory with the actual label values
pub(super) struct Label {
    /// Position in memory
    pub(super) pos: usize,
    pub(super) dtype: Type,
    pub(super) label: String,
}

/// Pushes a [Label](struct.Label.html) onto a vector and resizes the data accordingly
fn push_label(labels: &mut Vec<Label>, data: &mut Vec<u8>, dtype: Type, label: &str) {
    use Type::*;

    let pos = data.len();

    match dtype {
        Byte => data.resize(pos + 1, 0),
        Half => data.resize(pos + 2, 0),
        Word => data.resize(pos + 4, 0),
        _ => unreachable!("push_label should only be called with byte, half, or word directive"),
    }

    labels.push(Label {
        pos,
        dtype,
        label: label.to_owned(),
    });
}

// TODO: assert alignment
fn store_numerical(ctx: &mut ParserContext, token: &Token, value: u32) -> Result<(), ParserError> {
    use Type::*;
    match ctx.data_type {
        Byte => {
            ctx.data.push(value as u8);
        }
        Half => {
            let pos = ctx.data.len();
            ctx.data.resize(pos + 2, 0);
            LittleEndian::write_u16(&mut ctx.data[pos..], value as u16);
        }
        Word => {
            let pos = ctx.data.len();
            ctx.data.resize(pos + 4, 0);
            LittleEndian::write_u32(&mut ctx.data[pos..], value);
        }
        Float => {
            let pos = ctx.data.len();
            ctx.data.resize(pos + 4, 0);
            LittleEndian::write_f32(&mut ctx.data[pos..], f32::from_bits(value));
        }
        Align => {
            ctx.data.resize(ctx.data.len() + value as usize, 0);
        }
        _ => {
            return Err(
                ParserError::InvalidDataType(token.data.clone(), ctx.data_type)
                    .with_context(token.ctx.clone()),
            )
        }
    }

    Ok(())
}

pub fn push_data(token: Token, ctx: &mut ParserContext) -> Result<(), ParserError> {
    use super::token::Data::*;
    match token.data {
        Identifier(_label) => {
            unimplemented!("This version of FPGRARS does not support labels in .data")
        }
        Integer(i) => store_numerical(ctx, &token, i as u32)?,
        Float(f) => store_numerical(ctx, &token, f.to_bits())?,
        CharLiteral(c) => store_numerical(ctx, &token, c as u32)?,
        StringLiteral(s) => {
            ctx.data.extend(s.as_bytes());
            ctx.data.push(0);
            if ctx.data_type == Type::Asciz {
                ctx.data_type = Type::Byte;
            }
        }
        _ => unreachable!("push_data should only be called with a data token"),
    }
    Ok(())
}
