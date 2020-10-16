use nom::{
    self,
    branch::alt,
    bytes::complete::{escaped_transform, is_not, tag, take_till, take_till1},
    character::complete::{char as the_char, space0},
    combinator::{map, value},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

fn transform_escaped_char(c: &[u8]) -> IResult<&[u8], &[u8]> {
    alt((
        value("\\".as_bytes(), the_char('\\')),
        value("\"".as_bytes(), the_char('"')),
        value("\n".as_bytes(), the_char('n')),
        value("\t".as_bytes(), the_char('t')),
    ))(c)
}

pub fn quoted_string(s: &[u8]) -> IResult<&[u8], String> {
    let parser = delimited(
        the_char('"'),
        escaped_transform(is_not("\"\\"), '\\', transform_escaped_char),
        the_char('"'),
    );

    map(parser, |s: Vec<u8>| {
        std::str::from_utf8(&s).unwrap().to_owned()
    })(s)
}

#[test]
pub fn test_quoted_string() {
    assert_eq!(
        quoted_string("\"some quoted string\"".as_bytes()),
        Ok(("".as_bytes(), "some quoted string".to_owned()))
    );
    assert_eq!(
        quoted_string(r#""escape \"sequences\"\n parsed \t correctly""#.as_bytes()),
        Ok((
            "".as_bytes(),
            "escape \"sequences\"\n parsed \t correctly".to_owned()
        ))
    );
}

pub fn include_directive(s: &[u8]) -> IResult<&[u8], String> {
    let parser = tuple((space0, tag(".include"), space0, quoted_string));
    map(parser, |(_, _, _, file)| file)(s)
}

#[test]
pub fn test_include_directive() {
    assert_eq!(
        include_directive(".include \"some file.s\"".as_bytes()),
        Ok(("".as_bytes(), "some file.s".to_owned()))
    );
}

/// Strips indentation and removes comments
pub fn strip_unneeded(s: &str) -> Result<&str, nom::Err<(&str, nom::error::ErrorKind)>> {
    preceded(space0, take_till(|c| c == '#'))(s).map(|(_i, o)| o)
}

#[test]
pub fn test_strip_unneeded() {
    assert_eq!(
        strip_unneeded("     mv x0 x0 # does nothing"),
        Ok("mv x0 x0 "),
    );
}

/// Parses a line that *begins* with a label
pub fn parse_label(s: &str) -> IResult<&str, &str> {
    terminated(
        take_till1(|c| c == ':' || c == ' '),
        tuple((space0, tag(":"))),
    )(s)
}

#[test]
pub fn test_parse_label() {
    assert_eq!(parse_label("label: mv x0 x0"), Ok((" mv x0 x0", "label")),);
    assert_eq!(parse_label(".L0 : mv x0 x0"), Ok((" mv x0 x0", ".L0")),);
    assert_eq!(parse_label(": mv x0 x0").map_err(|_| ()), Err(()));
}
