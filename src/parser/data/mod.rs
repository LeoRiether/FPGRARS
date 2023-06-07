pub mod unlabel;
pub use unlabel::unlabel;

use crate::parser::error::Contextualize;
use crate::parser::LabelUseType;

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
            // `.align` Aligns the next data item along a specified byte boundary:
            // 0 = byte, 1 = half, 2 = word, 3 = double.
            let multiple = 1 << value;
            let blocks = (ctx.data.len() + multiple - 1) / multiple; // ceil(len / multiple)
            let len = blocks * multiple;
            ctx.data.resize(len, 0);
        }
    }

    Ok(())
}

/// Pushes a data token onto the data vector.
pub fn push_data(token: Token, ctx: &mut ParserContext) -> Result<(), Error> {
    use super::token::Data::*;
    match token.data {
        Identifier(label) => {
            let pos = ctx.use_label(&label, LabelUseType::Data, token.ctx.clone());
            store_numerical(ctx, pos)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alignment_manual() {
        let mut ctx = ParserContext {
            data_type: Type::Byte,
            ..Default::default()
        };

        assert!(store_numerical(&mut ctx, 123).is_ok());
        ctx.data_type = Type::Align;
        assert!(store_numerical(&mut ctx, 1).is_ok());
        assert_eq!(&ctx.data, &[123, 0]);
        assert!(store_numerical(&mut ctx, 2).is_ok());
        assert_eq!(&ctx.data, &[123, 0, 0, 0]);
        assert!(store_numerical(&mut ctx, 3).is_ok());
        assert_eq!(&ctx.data, &[123, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_alignment_empty_data() {
        let mut ctx = ParserContext {
            data_type: Type::Align,
            ..Default::default()
        };
        assert!(store_numerical(&mut ctx, 2).is_ok());
        assert_eq!(&ctx.data, &[]);
    }

    #[test]
    fn test_alignment() {
        let mut ctx = ParserContext::default();

        for align in 0..=4 {
            // Align the data
            ctx.data_type = Type::Align;
            assert!(store_numerical(&mut ctx, align).is_ok());

            for offset in 0..(1 << align) {
                // Store bytes to misalign the data
                ctx.data_type = Type::Byte;
                for _ in 0..offset {
                    assert!(store_numerical(&mut ctx, 0xFF).is_ok());
                }

                // Align the data
                ctx.data_type = Type::Align;
                assert!(store_numerical(&mut ctx, align).is_ok());

                // Make sure alignment is correct
                assert_eq!(
                    ctx.data.len() % (1 << align),
                    0,
                    "with align: {align}, offset: {offset}"
                );
            }
        }
    }
}
