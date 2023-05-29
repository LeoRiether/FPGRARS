pub mod unlabel;
pub use unlabel::unlabel;

use crate::parser::error::Contextualize;

use super::error::{Error, ParserError};
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
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Type::*;
        match s {
            "word" => Ok(Word),
            "byte" => Ok(Byte),
            "half" => Ok(Half),
            "align" | "space" => Ok(Align),
            "asciz" | "ascii" | "string" => Ok(Asciz),
            "float" => Ok(Float),
            _ => Err(ParserError::UnknownDirective(s.to_owned()).into()),
        }
    }
}

// TODO: assert alignment
/// Stores a numerical token with value `value` in the data vector.
fn store_numerical(ctx: &mut ParserContext, value: u32) -> Result<(), Error> {
    use Type::*;
    match ctx.data_type {
        Byte | Asciz => {
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
    }

    Ok(())
}

/// Pushes a data token onto the data vector.
pub fn push_data(token: Token, ctx: &mut ParserContext) -> Result<(), Error> {
    use super::token::Data::*;
    match token.data {
        Identifier(_label) => {
            unimplemented!("This version of FPGRARS does not support labels in .data")
        }
        Integer(i) => store_numerical(ctx, i as u32)?,
        Float(f) => store_numerical(ctx, f.to_bits())?,
        CharLiteral(c) => store_numerical(ctx, c as u32)?,
        StringLiteral(s) => {
            if ctx.data_type == Type::Asciz {
                ctx.data.extend(s.as_bytes());
                ctx.data.push(0);
            } else {
                return Err(
                    ParserError::InvalidDataType(StringLiteral(s), ctx.data_type)
                        .with_context(token.ctx),
                );
            }
        }
        _ => unreachable!("push_data should only be called with a data token"),
    }
    Ok(())
}
