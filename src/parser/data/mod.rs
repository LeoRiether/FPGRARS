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
    /// 32-bit word
    #[default]
    Word,
    /// 8-bit byte
    Byte,
    /// 16-bit halfword
    Half,
    /// Aligns the next data item along a specified byte boundary:
    /// 0 = byte, 1 = half, 2 = word, 3 = double.
    Align,
    /// Reserves space for `n` bytes
    Space,
    /// Null-terminated string
    Asciz,
    /// String that is not null-terminated
    Ascii,
    /// 32-bit floating point number
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
            "align" => Ok(Align),
            "space" => Ok(Space),
            "asciz" | "string" => Ok(Asciz),
            "ascii" => Ok(Ascii),
            "float" => Ok(Float),
            _ => Err(ParserError::UnknownDirective(s.to_owned()).into()),
        }
    }
}

/// Inserts padding into `data` so it's aligned to `alignment`.
/// alignment 0 = byte, 1 = half, 2 = word, 3 = double.
fn align(data: &mut Vec<u8>, alignment: u32) {
    let multiple = 1 << alignment;
    let blocks = (data.len() + multiple - 1) / multiple; // ceil(len / multiple)
    let len = blocks * multiple;
    data.resize(len, 0);
}

/// Stores a numerical token with value `value` in the data vector.
fn store_numerical(ctx: &mut ParserContext, value: u32) -> Result<(), Error> {
    use Type::*;

    // Align the data to the correct boundary
    match ctx.data_type {
        Half => align(&mut ctx.data, 1),
        Word | Float => align(&mut ctx.data, 2),
        Align => align(&mut ctx.data, value),
        Byte | Ascii | Asciz | Space => {}
    }

    // Commit the current data position to the label
    ctx.commit_data_label_backlog();

    // Push data into the data vector
    match ctx.data_type {
        Byte | Ascii | Asciz => {
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
        Space => ctx.data.resize(ctx.data.len() + value as usize, 0),
        Align => {}
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
            ctx.commit_data_label_backlog();
            if let Type::Asciz | Type::Ascii = ctx.data_type {
                ctx.data.extend(s.as_bytes());
                if let Type::Asciz = ctx.data_type {
                    ctx.data.push(0);
                }
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
    use crate::{
        parser::{lexer::Lexer, parse_tokens, token::Data, Parsed},
        simulator::memory::DATA_SIZE,
    };

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

    #[test]
    fn test_strings() {
        // Ascii (.ascii, not null-terminated)
        let mut ctx = ParserContext {
            data_type: Type::Ascii,
            ..Default::default()
        };

        let token = Token::new(Data::StringLiteral("Hello world!".to_owned()));
        assert!(push_data(token, &mut ctx).is_ok());
        assert_eq!(&ctx.data, b"Hello world!");

        // Asciz (.asciz or .string, null-terminated)
        let mut ctx = ParserContext {
            data_type: Type::Asciz,
            ..Default::default()
        };

        let token = Token::new(Data::StringLiteral("Hello world!".to_owned()));
        assert!(push_data(token, &mut ctx).is_ok());
        assert_eq!(&ctx.data, b"Hello world!\0");
    }

    #[test]
    fn test_label_alignment() {
        let input = ".data
            .byte '1'
            Word: .word 1234 # aligned to 2 bits
            .byte '2'
            Half: .half 1234 # aligned to 1 bit
            .byte '3'
            Align: .align 3  # aligned to 3 bits
            End:

            .text
            la x0 Word
            la x0 Half
            la x0 Align
            la x0 End";

        let tokens = Lexer::from_content(String::from(input), "test_label_alignment").peekable();
        let Parsed { code, .. } = parse_tokens(tokens, DATA_SIZE).unwrap();

        use crate::instruction::Instruction::{Ecall, Li};
        assert_eq!(
            &code,
            &[
                Li(0, 4),
                Li(0, 10),
                Li(0, 16),
                Li(0, 16),
                Li(17, 10),
                Li(10, 0),
                Ecall
            ]
        )
    }
}
