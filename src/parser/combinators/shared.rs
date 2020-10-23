use nom::{
    self,
    branch::alt,
    bytes::complete::{escaped_transform, is_not, take_till, take_till1},
    character::complete::{char as the_char},
    combinator::{map, value},
    sequence::{delimited},
    IResult,
};

pub type NomErr<'a> = nom::Err<(&'a str, nom::error::ErrorKind)>;
pub use super::super::util::Error;

pub fn is_separator(c: char) -> bool {
    c == ',' || c.is_whitespace()
}

pub fn separator0(s: &str) -> IResult<&str, ()> {
    map(take_till(|c| !is_separator(c)), |_| ())(s)
}
pub fn separator1(s: &str) -> IResult<&str, ()> {
    map(take_till1(|c| !is_separator(c)), |_| ())(s)
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
pub fn quoted_char(s: &str) -> IResult<&str, char> {
    let parser = delimited(
        the_char('\''),
        escaped_transform(is_not("\'\\"), '\\', transform_escaped_char),
        the_char('\''),
    );

    map(parser, |c| c.chars().next().unwrap())(s)
}

#[cfg(test)]
mod tests {
    use super::*;

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