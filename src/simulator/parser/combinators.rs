use nom::{
    self,
    branch::alt,
    bytes::complete::{escaped_transform, is_not, tag, take_till, take_till1},
    character::complete::{char as the_char, hex_digit1, space0},
    combinator::{all_consuming, map, map_res, value},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

pub type NomErr<'a> = nom::Err<(&'a str, nom::error::ErrorKind)>;

use super::register_names::{RegMap, TryGetRegister};
use super::util::Error;

macro_rules! all_consuming_tuple {
    ($tup:expr) => {
        all_consuming(tuple($tup))
    };
}

pub fn is_separator(c: char) -> bool {
    c == ',' || c.is_whitespace()
}

pub fn separator0(s: &str) -> IResult<&str, ()> {
    map(take_till(|c| !is_separator(c)), |_| ())(s)
}

fn transform_escaped_char(c: &str) -> IResult<&str, &str> {
    alt((
        value("\\", the_char('\\')),
        value("\"", the_char('"')),
        value("\n", the_char('n')),
        value("\t", the_char('t')),
    ))(c)
}

pub fn quoted_string(s: &str) -> IResult<&str, String> {
    delimited(
        the_char('"'),
        escaped_transform(is_not("\"\\"), '\\', transform_escaped_char),
        the_char('"'),
    )(s)
}

// TODO: remove duplicated logic, this is almost the same as fn quoted_string
fn quoted_char(s: &str) -> IResult<&str, char> {
    let parser = delimited(
        the_char('\''),
        escaped_transform(is_not("\'\\"), '\\', transform_escaped_char),
        the_char('\''),
    );

    map(parser, |c| c.chars().next().unwrap())(s)
}

/// Parses `.include "file.s"` and outputs `file.s`
pub fn include_directive(s: &str) -> IResult<&str, String> {
    let parser = tuple((separator0, tag(".include"), separator0, quoted_string));
    map(parser, |(_, _, _, file)| file)(s)
}

/// Strips indentation and removes comments
pub fn strip_unneeded(s: &str) -> Result<&str, NomErr> {
    preceded(space0, take_till(|c| c == '#'))(s)
        .map(|(_i, o)| o)
        .map(|s| s.trim_end())
}

/// Parses a line that *begins* with a label
pub fn parse_label(s: &str) -> IResult<&str, &str> {
    terminated(
        take_till1(|c| c == ':' || c == ' '),
        tuple((space0, tag(":"), space0)),
    )(s)
}

/// Parses one argument from the input and the separators that follow it.
/// Should work correctly for immediates, for example `one_arg("-4(sp)")` should only parse `-4`.
pub fn one_arg(s: &str) -> IResult<&str, &str> {
    terminated(take_till1(|c| is_separator(c) || c == '('), separator0)(s)
}

/// Parses something like `0x10ab` or `-0xff` to a u32.
/// For some reason, I can't get nom::number::complete::hex_u32 to work...
fn hex_immediate(s: &str) -> IResult<&str, u32> {
    alt((
        map_res(preceded(tag("0x"), hex_digit1), |x| {
            u32::from_str_radix(x, 16)
        }),
        map_res(preceded(tag("-0x"), hex_digit1), |x| {
            u32::from_str_radix(x, 16).map(|x| (!x).wrapping_add(1))
        }),
    ))(s)
}

/// Parses an immediate u32, i32 or char.
/// For example, in `li a7 100`, the last argument is the "immediate" 100
fn immediate(s: &str) -> IResult<&str, u32> {
    alt((
        // numeric immediate
        map_res(one_arg, |token| {
            all_consuming(hex_immediate)(token)
                .map(|(_, x)| x)
                .or_else(|_| token.parse::<u32>())
                .or_else(|_| token.parse::<i32>().map(|x| x as u32))
        }),
        // character immediate
        map(quoted_char, |c| c as u32),
    ))(s)
}

fn immediate_with_sep(s: &str) -> IResult<&str, u32> {
    terminated(immediate, separator0)(s)
}

/// Parses the arguments for a Type R instruction.
/// Expects the input without any separators in the prefix! For example:
/// `args_type_r("a0, a1, a2")`
pub fn args_type_r(s: &str, regs: &RegMap) -> Result<(u8, u8, u8), Error> {
    // TODO: make sure there are no trailing arguments
    let res = all_consuming_tuple!((
        one_arg, // rd
        one_arg, // rs1
        one_arg, // rs2
    ))(s);

    // Map the register names to their indices
    let (_i, (rd, rs1, rs2)) = res?;
    let (rd, rs1, rs2) = (regs.try_get(rd)?, regs.try_get(rs1)?, regs.try_get(rs2)?);
    Ok((rd, rs1, rs2))
}

/// Parses the arguments for a `jal`.
pub fn args_jal(s: &str, regs: &RegMap) -> Result<(u8, String), Error> {
    let res = all_consuming_tuple!((
        one_arg, // rd
        one_arg, // label
    ))(s);

    let (_i, (rd, label)) = res?;
    let (rd, label) = (regs.try_get(rd)?, label.to_owned());
    Ok((rd, label))
}

/// Parses the arguments for a Type SB instruction, like `bge` or `blt`
pub fn args_type_sb(s: &str, regs: &RegMap) -> Result<(u8, u8, String), Error> {
    let res = all_consuming_tuple!((
        one_arg, // rs1
        one_arg, // rs2
        one_arg, // label
    ))(s);

    // Map the register names to their indices
    let (_i, (rs1, rs2, label)) = res?;
    let (rs1, rs2, label) = (regs.try_get(rs1)?, regs.try_get(rs2)?, label.to_owned());
    Ok((rs1, rs2, label))
}

/// Parses the arguments for a type I instruction, like `addi t0 t1 123`
pub fn args_type_i(s: &str, regs: &RegMap) -> Result<(u8, u8, u32), Error> {
    let res = all_consuming_tuple!((
        one_arg, // rd
        one_arg, // rs1
        immediate_with_sep,
    ))(s);

    let (_i, (rd, rs1, imm)) = res?;
    let (rd, rs1) = (regs.try_get(rd)?, regs.try_get(rs1)?);
    Ok((rd, rs1, imm))
}

pub fn args_li(s: &str, regs: &RegMap) -> Result<(u8, u32), Error> {
    let res = all_consuming_tuple!((
        one_arg, // rd
        immediate_with_sep,
    ))(s);

    let (_i, (rd, imm)) = res?;
    let rd = regs.try_get(rd)?;
    Ok((rd, imm))
}

pub fn args_type_s(s: &str, regs: &RegMap) -> Result<(u8, u32, u8), Error> {
    let res = all_consuming_tuple!((
        one_arg,
        immediate_with_sep,
        delimited(
            the_char('('),
            delimited(
                separator0,
                take_till1(|c| is_separator(c) || c == ')'),
                separator0
            ),
            the_char(')')
        ),
        separator0,
    ))(s);

    let (_i, (r1, imm, r2, _)) = res?;
    let (r1, r2) = (regs.try_get(r1)?, regs.try_get(r2)?);
    Ok((r1, imm, r2))
}

pub fn args_mv(s: &str, regs: &RegMap) -> Result<(u8, u8), Error> {
    let res = all_consuming_tuple!((one_arg, one_arg))(s);

    let (_i, (rd, rs1)) = res?;
    let (rd, rs1) = (regs.try_get(rd)?, regs.try_get(rs1)?);
    Ok((rd, rs1))
}

#[cfg(test)]
mod tests {
    use super::super::register_names as reg_names;
    use super::*;

    use lazy_static::*;

    lazy_static! {
        static ref REGS: RegMap = reg_names::regs();
        static ref FLOATS: RegMap = reg_names::floats();
        static ref STATUS: RegMap = reg_names::status();
    }

    #[test]
    fn test_quoted_string() {
        assert_eq!(
            quoted_string("\"some quoted string\""),
            Ok(("", "some quoted string".to_owned()))
        );
        assert_eq!(
            quoted_string(r#""escape \"sequences\"\n parsed \t correctly""#),
            Ok(("", "escape \"sequences\"\n parsed \t correctly".to_owned()))
        );
    }

    #[test]
    fn test_include_directive() {
        assert_eq!(
            include_directive(".include \"some file.s\""),
            Ok(("", "some file.s".to_owned()))
        );
    }

    #[test]
    fn test_strip_unneeded() {
        assert_eq!(
            strip_unneeded("     mv x0 x0 # does nothing"),
            Ok("mv x0 x0"),
        );
        assert_eq!(strip_unneeded("j main # Does nothing"), Ok("j main"),)
    }

    #[test]
    fn test_parse_label() {
        assert_eq!(parse_label("label: mv x0 x0"), Ok(("mv x0 x0", "label")),);
        assert_eq!(parse_label(".L0 : mv x0 x0"), Ok(("mv x0 x0", ".L0")),);
        assert_eq!(parse_label(": mv x0 x0").map_err(|_| ()), Err(()));
    }

    #[test]
    fn test_one_arg() {
        assert_eq!(one_arg("li a7 10"), Ok(("a7 10", "li")));
        assert_eq!(one_arg("ecall"), Ok(("", "ecall")));
        assert_eq!(one_arg("mv, x0, x0"), Ok(("x0, x0", "mv")));
        assert_eq!(one_arg("something else"), Ok(("else", "something")));
    }

    #[test]
    fn test_immediate() {
        assert_eq!(immediate("123"), Ok(("", 123)));
        assert_eq!(immediate("-10"), Ok(("", (-10i32) as u32)));
        assert_eq!(immediate("0xff000000"), Ok(("", 4278190080)));
        assert_eq!(immediate("-0xff000000"), Ok(("", !(0xff000000) + 1)));
        assert_eq!(immediate("' '"), Ok(("", 32)));
        assert_eq!(immediate("'\\n'"), Ok(("", '\n' as u32)));
        assert_eq!(immediate("0xA(sp)"), Ok(("(sp)", 10)));
    }

    #[test]
    fn test_args_type_r() {
        assert_eq!(
            args_type_r("x0 x1 x2", &REGS).map_err(|_| ()),
            Ok((0, 1, 2))
        );
        assert_eq!(
            args_type_r("zero ra sp", &REGS).map_err(|_| ()),
            Ok((0, 1, 2))
        );
        assert_eq!(
            args_type_r("a0,,,a1 , a2 ,", &REGS).map_err(|_| ()),
            Ok((10, 11, 12))
        );
        assert_eq!(
            args_type_r("t0 t0 t1", &REGS).map_err(|_| ()),
            Ok((5, 5, 6))
        );
    }

    #[test]
    fn test_args_jal() {
        assert_eq!(
            args_jal("ra, some_label", &REGS).map_err(|_| ()),
            Ok((1, "some_label".to_owned()))
        );
    }

    #[test]
    fn test_args_type_sb() {
        assert_eq!(
            args_type_sb("s11 s10 LaBeL", &REGS).map_err(|_| ()),
            Ok((27, 26, "LaBeL".to_owned()))
        );
    }

    #[test]
    fn test_args_type_i() {
        assert_eq!(
            args_type_i("sp sp -4", &REGS).map_err(|_| ()),
            Ok((2, 2, (-4i32) as u32))
        );
        assert_eq!(
            args_type_i("a0, a0, 0x01,,", &REGS).map_err(|_| ()),
            Ok((10, 10, 1))
        );
    }

    #[test]
    fn test_args_type_s() {
        assert_eq!(
            args_type_s("x31 0xA(x25)", &REGS).map_err(|e| format!("{:?}", e)),
            Ok((31, 10, 25))
        );
        assert_eq!(
            args_type_s("x10 4 ( ,sp, )", &REGS).map_err(|_| ()),
            Ok((10, 4, 2))
        );
        assert_eq!(
            args_type_s("x0, ' ',(,x7,) ,", &REGS).map_err(|_| ()),
            Ok((0, 32, 7))
        );
        assert_eq!(
            args_type_s("x0 -1(zero)", &REGS).map_err(|_| ()),
            Ok((0, (-1i32) as u32, 0))
        );
    }
}
