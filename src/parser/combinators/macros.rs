use nom::{
    self,
    branch::alt,
    bytes::complete::{tag, take_till1},
    character::complete::{char as the_char},
    combinator::{all_consuming, map},
    multi::separated_list,
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

use super::shared::*;
use super::riscv::owned_one_arg;

fn macro_tag(s: &str) -> IResult<&str, ()> {
    map(terminated(tag(".macro"), separator1), |_| ())(s)
}

fn macro_arg_list(s: &str) -> IResult<&str, Vec<String>> {
    separated_list(
        separator1,
        preceded(
            the_char('%'),
            map(take_till1(|c| is_separator(c) || c == ')'), str::to_owned),
        ),
    )(s)
}

/// Parses `.macro NAME(%arg1, %arg2)` into `("NAME", ["arg1", "arg2"])`
pub fn declare_macro(s: &str) -> IResult<&str, (String, Vec<String>)> {
    preceded(
        macro_tag,
        tuple((
            owned_one_arg, // name
            alt((
                delimited(the_char('('), macro_arg_list, the_char(')')), // parenthesis enclosed args
                map(all_consuming(separator0), |_| vec![]), // no args
            )),
        )),
    )(s)
}

/// Recognizes`.end_macro`
pub fn end_macro(s: &str) -> bool {
    all_consuming(terminated(tag(".end_macro"), separator0))(s).is_ok()
}

pub fn declare_eqv(s: &str) -> IResult<&str, (String, String)> {
    preceded(
        delimited(separator0, tag(".eqv"), separator1),
        tuple((owned_one_arg, owned_one_arg))
    )(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_declare_macro() {
        assert_eq!(
            declare_macro(".macro NAME(%arg1, %arg2)").map_err(|_| ()),
            Ok(("", ("NAME".into(), vec!["arg1".into(), "arg2".into()])))
        );
        assert_eq!(
            declare_macro(".macro NAME").map_err(|_| ()),
            Ok(("", ("NAME".into(), vec![])))
        );
        assert_eq!(
            declare_macro(".macro MV(%rd %rs1)").map_err(|_| ()),
            Ok(("", ("MV".into(), vec!["rd".into(), "rs1".into()])))
        );
    }

    #[test]
    fn test_declare_eqv() {
        assert_eq!(
            declare_eqv(".eqv SCREEN_START 0xFF000000"),
            Ok(("", ("SCREEN_START".into(), "0xFF000000".into())))
        );
    }
}