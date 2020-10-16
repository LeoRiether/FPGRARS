use nom::{
    self,
    branch::alt,
    bytes::complete::{escaped_transform, is_a, is_not, tag, take_till, take_till1},
    character::complete::{char as the_char, one_of, space0},
    combinator::{map, map_res, value},
    multi::{many1, separated_list},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

pub type NomErr<'a> = nom::Err<(&'a str, nom::error::ErrorKind)>;

use super::register_names::{RegMap, TryGetRegister};
use super::util::Error;

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

pub fn include_directive(s: &str) -> IResult<&str, String> {
    let parser = tuple((space0, tag(".include"), space0, quoted_string));
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

pub fn is_separator(c: char) -> bool {
    c == ',' || c.is_whitespace()
}

pub fn separator1(s: &str) -> IResult<&str, ()> {
    map(take_till1(|c| !is_separator(c)), |_| ())(s)
}

pub fn separator0(s: &str) -> IResult<&str, ()> {
    map(take_till(|c| !is_separator(c)), |_| ())(s)
}

/// Parses one argument from the input and the separators that follow it.
pub fn one_arg(s: &str) -> IResult<&str, &str> {
    terminated(take_till1(is_separator), separator0)(s)
}

/// Parses the arguments for a Type R instruction.
/// Expects the input without any separators in the prefix! For example:
/// `args_type_r("a0, a1, a2")`
pub fn args_type_r(s: &str, regs: &RegMap) -> Result<(u8, u8, u8), Error> {
    // TODO: make sure there are no trailing arguments
    let res = tuple((
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
    let res = tuple((
        one_arg, // rd
        one_arg, // label
    ))(s);

    let (_i, (rd, label)) = res?;
    let (rd, label) = (regs.try_get(rd)?, label.to_owned());
    Ok((rd, label))
}

/// Parses the arguments for a Type SB instruction, like `bge` or `blt`
pub fn args_type_sb(s: &str, regs: &RegMap) -> Result<(u8, u8, String), Error> {
    let res = tuple((
        one_arg, // rs1
        one_arg, // rs2
        one_arg, // label
    ))(s);

    // Map the register names to their indices
    let (_i, (rs1, rs2, label)) = res?;
    let (rs1, rs2, label) = (regs.try_get(rs1)?, regs.try_get(rs2)?, label.to_owned());
    Ok((rs1, rs2, label))
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
    fn test_separator1() {
        assert_eq!(separator1(" ,  , ,,, "), Ok(("", ())));
        assert_eq!(separator1("  ,,, , li t0, 123"), Ok(("li t0, 123", ())));
    }

    #[test]
    fn test_one_arg() {
        assert_eq!(one_arg("li a7 10"), Ok(("a7 10", "li")));
        assert_eq!(one_arg("ecall"), Ok(("", "ecall")));
        assert_eq!(one_arg("mv, x0, x0"), Ok(("x0, x0", "mv")));
        assert_eq!(one_arg("something else"), Ok(("else", "something")));
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
    }

    #[test]
    fn test_args_jal() {
        assert_eq!(
            args_jal("ra, some_label", &REGS).map_err(|_| ()),
            Ok((1, "some_label".to_owned()))
        );
    }

    #[test]
    fn test_args_sb() {
        assert_eq!(
            args_type_sb("s11 s10 LaBeL", &REGS).map_err(|_| ()),
            Ok((27, 26, "LaBeL".to_owned()))
        );
    }
}
