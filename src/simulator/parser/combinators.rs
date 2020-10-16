use nom::{
    self,
    branch::alt,
    bytes::complete::{escaped_transform, is_not, tag, take_till, take_till1},
    character::complete::{char as the_char, space0},
    combinator::{map, value},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

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
pub fn strip_unneeded(s: &str) -> Result<&str, nom::Err<(&str, nom::error::ErrorKind)>> {
    preceded(space0, take_till(|c| c == '#'))(s)
        .map(|(_i, o)| o)
        .map(|s| s.trim_end())
}

/// Parses a line that *begins* with a label
pub fn parse_label(s: &str) -> IResult<&str, &str> {
    terminated(
        take_till1(|c| c == ':' || c == ' '),
        tuple((space0, tag(":"))),
    )(s)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_label() {
        assert_eq!(parse_label("label: mv x0 x0"), Ok((" mv x0 x0", "label")),);
        assert_eq!(parse_label(".L0 : mv x0 x0"), Ok((" mv x0 x0", ".L0")),);
        assert_eq!(parse_label(": mv x0 x0").map_err(|_| ()), Err(()));
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
    fn test_include_directive() {
        assert_eq!(
            include_directive(".include \"some file.s\""),
            Ok(("", "some file.s".to_owned()))
        );
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
}
