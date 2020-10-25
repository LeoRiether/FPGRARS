use nom::{
    self,
    branch::alt,
    bytes::complete::{tag, take_till1, take_while1},
    character::complete::char as the_char,
    combinator::{all_consuming, map},
    multi::separated_list,
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

use super::riscv::{one_arg, owned_one_arg};
use super::shared::*;

fn macro_tag(s: &str) -> IResult<&str, &str> {
    terminated(tag(".macro"), separator1)(s)
}

fn arg_list(s: &str) -> IResult<&str, Vec<String>> {
    separated_list(
        separator1,
        preceded(
            the_char('%'),
            map(take_till1(|c| is_separator(c) || c == ')'), str::to_owned),
        ),
    )(s)
}

fn parenthesized_arg_list(s: &str) -> IResult<&str, Vec<String>> {
    alt((
        delimited(the_char('('), arg_list, the_char(')')), // parenthesis enclosed args
        map(all_consuming(separator0), |_| vec![]),        // no args
    ))(s)
}

/// Parses `.macro NAME(%arg1, %arg2)` into `("NAME", ["arg1", "arg2"])`
pub fn declare_macro(s: &str) -> IResult<&str, (String, Vec<String>)> {
    preceded(
        macro_tag,
        tuple((
            owned_one_arg, // name
            parenthesized_arg_list,
        )),
    )(s)
}

/// Recognizes`.end_macro`
pub fn end_macro(s: &str) -> bool {
    all_consuming(terminated(tag(".end_macro"), separator0))(s).is_ok()
}

fn use_arg_list(s: &str) -> IResult<&str, Vec<String>> {
    separated_list(
        separator1,
        map(take_till1(|c| is_separator(c) || c == ')'), str::to_owned),
    )(s)
}

/// Parses either `MACRO(...args)` or `MACRO`. Note that this identifies a line like `nop` as
/// a macro with no arguments, but if the macro "nop" hasn't been declared, the macro parser
/// will not identify this as a macro usage and pass the line onwards to the riscv parser
pub fn macro_use(s: &str) -> IResult<&str, (String, Vec<String>)> {
    let res = all_consuming(delimited(
        separator0,
        alt((
            // MACRO(...args)
            tuple((
                one_arg,
                delimited(the_char('('), use_arg_list, the_char(')')),
            )),
            // MACRO
            map(one_arg, |a| (a, vec![])),
        )),
        separator0,
    ))(s);

    let (i, (name, args)) = res?;
    Ok((i, (name.to_owned(), args)))
}

pub fn declare_eqv(s: &str) -> IResult<&str, (String, String)> {
    preceded(
        delimited(separator0, tag(".eqv"), separator1),
        tuple((
            owned_one_arg,
            map(take_while1(|_| true), |tok: &str| tok.trim_end().to_owned()),
        )),
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
    fn test_macro_use() {
        assert_eq!(
            macro_use("    DO(mv x1 x0)"),
            Ok((
                "",
                ("DO".into(), vec!["mv".into(), "x1".into(), "x0".into()])
            )),
        );
        assert_eq!(macro_use(" MACRO "), Ok(("", ("MACRO".into(), vec![]))));
        assert_eq!(macro_use(" MACRO()"), Ok(("", ("MACRO".into(), vec![]))));

        // Parsing the label is the job of MacroParser::parse_macro_use
        // because we need to keep it and pass it to the riscv parser
        assert_eq!(
            macro_use("label: DE1(s8,Label.L0)").is_err(),
            true
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
