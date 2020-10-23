use nom::{
    self,
    bytes::complete::{tag, take_till},
    character::complete::space0,
    sequence::{delimited, preceded},
    IResult,
};

use super::shared::*;

fn include_tag(s: &str) -> IResult<&str, &str> {
    delimited(separator0, tag(".include"), separator0)(s)
}

/// Parses `.include "file.s"` and outputs `file.s`
pub fn include_directive(s: &str) -> IResult<&str, String> {
    preceded(include_tag, quoted_string)(s)
}

/// Strips indentation and removes comments
pub fn strip_unneeded(s: &str) -> Result<&str, NomErr> {
    preceded(space0, take_till(|c| c == '#'))(s)
        .map(|(_i, o)| o)
        .map(|s| s.trim_end())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_include_directive() {
        assert_eq!(
            include_directive(".include \"some file.s\""),
            Ok(("", "some file.s".to_owned()))
        );
        assert_eq!(
            include_directive(r#"  .include"file\n.s"  "#),
            Ok(("  ", "file\n.s".to_owned()))
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
}
