use super::{combinators::*, util::Error};

use nom::{
    bytes::complete::take_till1, character::complete::char as the_char, multi::separated_list,
    sequence::preceded,
};

use byteorder::{ByteOrder, LittleEndian};
use std::borrow::{Borrow, Cow};
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub(super) enum Type {
    Word,
    Byte,
    Half,
    Align,
    Asciz,
    Float,
}

impl Default for Type {
    fn default() -> Type {
        Type::Word
    }
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
            _ => Err(Error::UnrecognizedDataType(s.to_owned())),
        }
    }
}

fn store_integer(x: u32, data: &mut Vec<u8>, dtype: Type) {
    use Type::*;
    match dtype {
        Byte => {
            data.push(x as u8);
        }
        Half => {
            let pos = data.len();
            data.resize(pos + 2, 0);
            LittleEndian::write_u16(&mut data[pos..], x as u16);
        }
        Word => {
            let pos = data.len();
            data.resize(pos + 4, 0);
            LittleEndian::write_u32(&mut data[pos..], x as u32);
        }
        Align => {
            data.resize(data.len() + x as usize, 0);
        }
        _ => unreachable!("store_integer should only be called with an integer dtype"),
    }
}

fn store_token(s: &str, data: &mut Vec<u8>, dtype: Type) -> Result<(), Error> {
    use Type::*;
    match dtype {
        Byte | Half | Word | Align => {
            let (_, x) = immediate(s)?;
            store_integer(x, data, dtype);
        }
        Float => {
            let x = match s.parse::<f32>() {
                Ok(x) => x,
                Err(e) => return Err(Error::FloatError(e)),
            };

            let pos = data.len();
            data.resize(pos + 4, 0);
            LittleEndian::write_f32(&mut data[pos..], x);
        }
        Asciz => {
            data.extend(s.bytes().chain(Some(b'\0')));
        }
    }

    Ok(())
}

fn directive_to_type(s: &str) -> Result<(&str, Type), Error> {
    let (i, dir_str) = preceded(the_char('.'), one_arg)(s)?;

    dir_str.parse::<Type>().map(move |dtype| (i, dtype))
}

fn one_token(dtype: Type) -> impl Fn(&str) -> nom::IResult<&str, Cow<str>> {
    move |s: &str| {
        use Type::*;
        match dtype {
            Word | Byte | Half | Align | Float => {
                let (i, parsed) = take_till1(|c| is_separator(c))(s)?;
                Ok((i, Cow::from(parsed)))
            }
            Asciz => {
                let (i, parsed) = quoted_string(s)?;
                Ok((i, Cow::from(parsed)))
            }
        }
    }
}

/// Parses a line in the `.data` directive, puts the desired vales in `data` and
/// updates the `type` parameter.
pub(super) fn parse_line(s: &str, data: &mut Vec<u8>, dtype: &mut Type) -> Result<(), Error> {
    let (s, opt_new_dtype) = match directive_to_type(s) {
        Ok((rest, new_dtype)) => (rest, Some(new_dtype)),
        Err(_) => (s, None),
    };

    if let Some(new_dtype) = opt_new_dtype {
        *dtype = new_dtype;
    }

    let (_i, tokens) = separated_list(separator1, one_token(*dtype))(s)?;

    for tok in tokens {
        store_token(tok.borrow(), data, *dtype)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_line() {}
}
