use nom::{
    self,
    branch::alt,
    bytes::complete::{tag, take_till1},
    character::complete::{char as the_char, hex_digit1},
    combinator::{all_consuming, map, map_res, opt},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

use super::shared::*;
use crate::parser::register_names::{RegMap, TryGetRegister};

macro_rules! all_consuming_tuple {
    ($tup:expr) => {
        all_consuming(tuple($tup))
    };
}

/// Parses a line that *begins* with a label
pub fn parse_label(s: &str) -> IResult<&str, &str> {
    terminated(
        take_till1(|c| c == ':' || is_separator(c)),
        tuple((separator0, the_char(':'), separator0)),
    )(s)
}

/// Parses one argument from the input and the separators that follow it.
/// Should work correctly for immediates, for example `one_arg("-4(sp)")` should only parse `-4`.
pub fn one_arg(s: &str) -> IResult<&str, &str> {
    terminated(take_till1(|c| is_separator(c) || c == '('), separator0)(s)
}

pub fn owned_one_arg(s: &str) -> IResult<&str, String> {
    map(one_arg, str::to_owned)(s)
}

pub fn one_reg<'a>(regs: &'a RegMap) -> impl Fn(&'a str) -> IResult<&str, u8> {
    move |s: &'a str| map_res(one_arg, move |r| regs.try_get(r))(s)
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
pub fn immediate(s: &str) -> IResult<&str, u32> {
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

fn opt_immediate_with_sep(s: &str) -> IResult<&str, u32> {
    map(opt(immediate_with_sep), |x| x.unwrap_or(0))(s)
}

/// Parses the arguments for a Type R instruction.
/// Expects the input without any separators in the prefix! For example:
/// `args_type_r("a0, a1, a2")`
pub fn args_type_r(s: &str, regs: &RegMap) -> Result<(u8, u8, u8), Error> {
    let (_i, out) = all_consuming_tuple!((
        one_reg(regs), // rd
        one_reg(regs), // rs1
        one_reg(regs), // rs2
    ))(s)?;

    Ok(out)
}

/// Parses the arguments for a `jal`.
pub fn args_jal(s: &str, regs: &RegMap) -> Result<(u8, String), Error> {
    let (_i, out) = all_consuming_tuple!((
        one_reg(regs), // rd
        owned_one_arg,
    ))(s)?;

    Ok(out)
}

/// Parses the arguments for a Type SB instruction, like `bge` or `blt`
pub fn args_type_sb(s: &str, regs: &RegMap) -> Result<(u8, u8, String), Error> {
    let (_i, out) = all_consuming_tuple!((
        one_reg(regs), // rs1
        one_reg(regs), // rs2
        owned_one_arg, // label
    ))(s)?;

    Ok(out)
}

/// Parses the arguments for a type I instruction, like `addi t0 t1 123`
pub fn args_type_i(s: &str, regs: &RegMap) -> Result<(u8, u8, u32), Error> {
    let (_i, out) = all_consuming_tuple!((
        one_reg(regs), // rd
        one_reg(regs), // rs1
        immediate_with_sep,
    ))(s)?;

    Ok(out)
}

pub fn args_li(s: &str, regs: &RegMap) -> Result<(u8, u32), Error> {
    let (_i, out) = all_consuming_tuple!((
        one_reg(regs), // rd
        immediate_with_sep,
    ))(s)?;

    Ok(out)
}

pub fn args_type_s_mixed(s: &str, rs2_regs: &RegMap, rs1_regs: &RegMap) -> Result<(u8, u32, u8), Error> {
    let (_i, out) = all_consuming_tuple!((
        one_reg(rs2_regs),
        opt_immediate_with_sep,
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
    ))(s)?;

    let (r1, imm, r2, _) = out;
    let r2 = rs1_regs.try_get(r2)?;
    Ok((r1, imm, r2))
}

pub fn args_type_s(s: &str, regs: &RegMap) -> Result<(u8, u32, u8), Error> {
    args_type_s_mixed(s, regs, regs)
}

pub fn args_mv(s: &str, regs: &RegMap) -> Result<(u8, u8), Error> {
    let (_i, out) = all_consuming_tuple!((one_reg(regs), one_reg(regs)))(s)?;
    Ok(out)
}

pub fn args_csr_small(s: &str, regs: &RegMap, status: &RegMap) -> Result<(u8, u8), Error> {
    let (_i, out) = all_consuming_tuple!((
        one_reg(regs),   // rs1
        one_reg(status)  // fcsr
    ))(s)?;

    Ok(out)
}
pub fn args_csr(s: &str, regs: &RegMap, status: &RegMap) -> Result<(u8, u8, u8), Error> {
    let (_i, out) = all_consuming_tuple!((
        one_reg(regs),   // rd
        one_reg(status), // fcsr
        one_reg(regs)    // rs1
    ))(s)?;

    Ok(out)
}
pub fn args_csr_small_imm(s: &str, status: &RegMap) -> Result<(u8, u32), Error> {
    let (_i, out) = all_consuming_tuple!((
        one_reg(status), // fcsr
        immediate_with_sep
    ))(s)?;

    Ok(out)
}
pub fn args_csr_imm(s: &str, regs: &RegMap, status: &RegMap) -> Result<(u8, u8, u32), Error> {
    let (_i, out) = all_consuming_tuple!((
        one_reg(regs),   // rd
        one_reg(status), // fcsr
        immediate_with_sep
    ))(s)?;

    Ok(out)
}

/// Parses the args for a `sw t0 label t1`, which stores `t0` in the address of `label`
/// using `t1` as a temporary
pub fn args_multi_store(s: &str, regs: &RegMap) -> Result<(u8, String, u8), Error> {
    let (_i, out) = all_consuming_tuple!((
        one_reg(regs), // rs2
        owned_one_arg, // label
        one_reg(regs)  // temporary
    ))(s)?;

    Ok(out)
}

pub fn args_float_r_mixed(s: &str, regs: &RegMap, floats: &RegMap) -> Result<(u8, u8, u8), Error> {
    let (_i, out) = all_consuming(terminated(
        tuple((
            one_reg(regs), // rd
            one_reg(floats), // rs1
            one_reg(floats), // rs2
        )),
        opt(one_arg), // rounding mode
    ))(s)?;

    Ok(out)
}

/// Almost the same as args_type_r, but accepts a rounding mode at the end
/// (and ignores it)
pub fn args_float_r(s: &str, floats: &RegMap) -> Result<(u8, u8, u8), Error> {
    args_float_r_mixed(s, floats, floats)
}

pub fn float_two_regs(s: &str, rd_regs: &RegMap, rs1_regs: &RegMap) -> Result<(u8, u8), Error> {
    let (_i, out) = all_consuming(terminated(
        tuple((
            one_reg(rd_regs), // rd
            one_reg(rs1_regs), // rs1
        )),
        opt(one_arg), // rounding mode
    ))(s)?;

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::register_names::{self as reg_names, RegMap};
    use lazy_static::*;

    lazy_static! {
        static ref REGS: RegMap = reg_names::regs();
        static ref FLOATS: RegMap = reg_names::floats();
        static ref STATUS: RegMap = reg_names::status();
    }

    #[test]
    fn test_parse_label() {
        assert_eq!(parse_label("label: mv x0 x0"), Ok(("mv x0 x0", "label")),);
        assert_eq!(parse_label(".L0 : mv x0 x0"), Ok(("mv x0 x0", ".L0")),);
        assert_eq!(parse_label(": mv x0 x0").map_err(|_| ()), Err(()));
    }

    #[test]
    fn test_one_arg() {
        assert_eq!(one_arg("li\t a7 10"), Ok(("a7 10", "li")));
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
        assert_eq!(
            // Why is this a thing?
            args_type_s("t1 (t0)", &REGS).map_err(|_| ()),
            Ok((6, 0, 5))
        );
    }

    #[test]
    fn test_args_csr() {
        assert_eq!(
            args_csr_small("x15 time", &REGS, &STATUS).map_err(|_| ()),
            Ok((15, STATUS.get("time").copied().unwrap()))
        );
        assert_eq!(
            // why would you csrr ra instret
            args_csr_small("ra instret", &REGS, &STATUS).map_err(|_| ()),
            Ok((1, STATUS.get("instret").copied().unwrap()))
        );
        assert_eq!(
            args_csr("x15 time x0", &REGS, &STATUS).map_err(|_| ()),
            Ok((15, STATUS.get("time").copied().unwrap(), 0))
        );
        assert_eq!(
            // why would you csrr ra instret
            args_csr("ra instret, sp", &REGS, &STATUS).map_err(|_| ()),
            Ok((1, STATUS.get("instret").copied().unwrap(), 2))
        );
    }

    #[test]
    fn test_args_float_r() {
        assert_eq!(
            args_float_r("ft0 ft1 ft2", &FLOATS).map_err(|_| ()),
            Ok((0, 1, 2))
        );
        assert_eq!(
            args_float_r("ft0 ft1 ft2 dyn", &FLOATS).map_err(|_| ()),
            Ok((0, 1, 2))
        );
    }
}
