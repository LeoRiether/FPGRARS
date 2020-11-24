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

/// Stores the information of a label we found in the `.data` directive, so we can
/// later populate the memory with the actual label values
pub(super) struct Label {
    /// Position in memory
    pub(super) pos: usize,
    pub(super) dtype: Type,
    pub(super) label: String,
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

fn store_token(
    s: &str,
    data: &mut Vec<u8>,
    found_labels: &mut Vec<Label>,
    dtype: Type,
) -> Result<(), Error> {
    use Type::*;
    match dtype {
        Byte | Half | Word => match immediate(s) {
            // .word <immediate>
            Ok((_, x)) => store_integer(x, data, dtype),

            // might be a .word <label>, might be .word <junk>
            Err(_) => push_label(found_labels, data, dtype, s),
        },
        Align => {
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
/// If we find something that could be a label, we should store a [Label](struct.Label.html)
/// so we can calculate the value to put in that position after parsing has been completed.
pub(super) fn parse_line(
    s: &str,
    data: &mut Vec<u8>,
    found_labels: &mut Vec<Label>,
    dtype: &mut Type,
) -> Result<(), Error> {
    let (s, opt_new_dtype) = match directive_to_type(s) {
        Ok((rest, new_dtype)) => (rest, Some(new_dtype)),
        Err(_) => (s, None),
    };

    if let Some(new_dtype) = opt_new_dtype {
        *dtype = new_dtype;
    }

    let (_i, tokens) = separated_list(separator1, one_token(*dtype))(s)?;

    for tok in tokens {
        store_token(tok.borrow(), data, found_labels, *dtype)?;
    }

    Ok(())
}
