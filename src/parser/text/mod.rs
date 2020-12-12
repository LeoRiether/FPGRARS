use super::{
    combinators::*,
    register_names::{FullRegMap, RegMap},
    util::Error,
    FloatInstruction, Instruction, PreLabelInstruction,
};

/// Parses a line that produces many instructions at a time, like `lw a0 label`.
pub(super) fn parse_multi_instruction(s: &str, regmaps: &FullRegMap) -> Option<Vec<PreLabelInstruction>> {
    let (regs, _floats, _status) = regmaps;

    use PreLabelInstruction as pre;
    use Instruction::*;

    let (s, instruction) = match one_arg(s) {
        Ok((s, i)) => (s, i),
        Err(_) => { return None; }
    };

    macro_rules! load {
        ($inst:ident) => {
            args_jal(s, &regs)
                .map(|(rd, label)| vec![
                    pre::La(rd, label),
                    $inst(rd, 0, rd).into(),
                ])
                .ok()
        }
    }

    macro_rules! store {
        ($inst:ident) => {
            args_multi_store(s, &regs)
                .map(|(rs2, label, tmp)| vec![
                    pre::La(tmp, label),
                    $inst(rs2, 0, tmp).into(),
                ])
                .ok()
        }
    }

    match instruction {
        "lb" => load!(Lb),
        "lh" => load!(Lh),
        "lw" => load!(Lw),
        "lbu" => load!(Lbu),
        "lhu" => load!(Lhu),

        "sb" => store!(Sb),
        "sh" => store!(Sh),
        "sw" => store!(Sw),
        _ => None
    }
}

/// Parses a line that produces a single instruction
pub(super) fn parse_instruction(s: &str, regmaps: &FullRegMap) -> Result<PreLabelInstruction, Error> {
    let (regs, floats, status) = regmaps;

    use FloatInstruction as F;
    use Instruction::*;
    use PreLabelInstruction as pre;

    let (s, instruction) = one_arg(s)?;

    macro_rules! type_i {
        ($inst:expr) => {
            args_type_i(s, &regs).map(|(rd, rs1, imm)| $inst(rd, rs1, imm).into())?
        };
    }

    macro_rules! type_r {
        ($inst:expr) => {
            args_type_r(s, &regs).map(|(rd, rs1, rs2)| $inst(rd, rs1, rs2).into())?
        };
        (float $inst:expr) => {
            args_float_r(s, &floats).map(|(rd, rs1, rs2)| $inst(rd, rs1, rs2).into())?
        };
        (mixed $inst:expr) => {
            args_float_r_mixed(s, &regs, &floats)
                .map(|(rd, rs1, rs2)| $inst(rd, rs1, rs2).into())?
        };
    }

    macro_rules! type_sb {
        ($inst:expr) => {
            args_type_sb(s, &regs).map(|(rs1, rs2, label)| $inst(rs1, rs2, label))?
        };
    }

    // bgez, bnez, ...
    macro_rules! type_sb_z {
        ($inst:expr) => {
            args_jal(s, &regs).map(|(rs1, label)| $inst(rs1, 0, label))?
        };
    }

    // Reverses the order of rs1 and rs2 to convert, for example,
    // `ble t0 t1 label` into `bge t1 t0 label`
    macro_rules! type_sb_reversed {
        ($inst:expr) => {
            args_type_sb(s, &regs).map(|(rs1, rs2, label)| $inst(rs2, rs1, label))?
        };
    }

    // blez, ...
    macro_rules! type_sb_reversed_z {
        ($inst:expr) => {
            args_jal(s, &regs).map(|(rs1, label)| $inst(0, rs1, label))?
        };
    }

    macro_rules! type_s {
        ($inst:expr) => {
            args_type_s(s, &regs).map(|(r1, imm, r2)| $inst(r1, imm, r2).into())?
        };
        (float $inst:expr) => {
            args_type_s_mixed(s, &floats, &regs).map(|(r1, imm, r2)| $inst(r1, imm, r2).into())?
        };
    }

    macro_rules! csr {
        ($inst:expr) => {
            args_csr(s, &regs, &status).map(|(rd, fcsr, rs1)| $inst(rd, fcsr, rs1).into())?
        };
    }

    macro_rules! csr_imm {
        ($inst:expr) => {
            args_csr_imm(s, &regs, &status).map(|(rd, fcsr, imm)| $inst(rd, fcsr, imm).into())?
        };
    }

    macro_rules! csr_small {
        ($inst:expr) => {
            args_csr_small(s, &regs, &status).map(|(rs1, fcsr)| $inst(0, fcsr, rs1).into())?
        };
    }

    macro_rules! csr_small_imm {
        ($inst:expr) => {
            args_csr_small_imm(s, &status).map(|(fcsr, imm)| $inst(0, fcsr, imm).into())?
        };
    }

    macro_rules! float_two_regs {
        ($inst:expr, $rd_regmap:expr, $rs1_regmap:expr) => {
            float_two_regs(s, $rd_regmap, $rs1_regmap).map(|(rd, rs1)| $inst(rd, rs1).into())?
        };
    }

    let parsed = match instruction.to_lowercase().as_str() {
        // Type R
        "add" => type_r!(Add),
        "sub" => type_r!(Sub),
        "sll" => type_r!(Sll),
        "slt" => type_r!(Slt),
        "sltu" => type_r!(Sltu),
        "xor" => type_r!(Xor),
        "srl" => type_r!(Srl),
        "sra" => type_r!(Sra),
        "or" => type_r!(Or),
        "and" => type_r!(And),
        "mul" => type_r!(Mul),
        "div" => type_r!(Div),
        "divu" => type_r!(Divu),
        "rem" => type_r!(Rem),
        "remu" => type_r!(Remu),
        "neg" => args_mv(s, &regs).map(|(rd, rs1)| Sub(rd, 0, rs1).into())?,
        "not" => args_mv(s, &regs).map(|(rd, rs1)| Xori(rd, rs1, (-1i32) as u32).into())?,
        "mv" => args_mv(s, &regs).map(|(rd, rs1)| Mv(rd, rs1).into())?,
        "snez" => args_mv(s, &regs).map(|(rd, rs1)| Sltu(rd, 0, rs1).into())?,
        "sltz" => args_mv(s, &regs).map(|(rd, rs1)| Slt(rd, rs1, 0).into())?,
        "sgtz" => args_mv(s, &regs).map(|(rd, rs1)| Slt(rd, 0, rs1).into())?,

        // Type I
        "addi" => type_i!(Addi),
        "slli" => type_i!(Slli),
        "slti" => type_i!(Slti),
        "sltiu" => type_i!(Sltiu),
        "xori" => type_i!(Xori),
        "srli" => type_i!(Srli),
        "srai" => type_i!(Srai),
        "ori" => type_i!(Ori),
        "andi" => type_i!(Andi),
        "jalr" => type_i!(Jalr),
        "jr" => one_reg(&regs)(s).map(|(_i, rs1)| Jalr(0, rs1, 0).into())?,
        "seqz" => args_mv(s, &regs).map(|(rd, rs1)| Sltiu(rd, rs1, 1).into())?,

        // Type I, loads from memory
        "lb" => type_s!(Lb),
        "lh" => type_s!(Lh),
        "lw" => type_s!(Lw),
        "lbu" => type_s!(Lbu),
        "lhu" => type_s!(Lhu),

        // Type S
        "sb" => type_s!(Sb),
        "sh" => type_s!(Sh),
        "sw" => type_s!(Sw),

        // Type SB and pseudoinstructions that map to SBs
        "beq" => type_sb!(pre::Beq),
        "bne" => type_sb!(pre::Bne),
        "blt" => type_sb!(pre::Blt),
        "bge" => type_sb!(pre::Bge),
        "bltu" => type_sb!(pre::Bltu),
        "bgeu" => type_sb!(pre::Bgeu),
        "bgt" => type_sb_reversed!(pre::Blt),
        "ble" => type_sb_reversed!(pre::Bge),
        "bgtu" => type_sb_reversed!(pre::Bltu),
        "bleu" => type_sb_reversed!(pre::Bgeu),
        "beqz" => type_sb_z!(pre::Beq),
        "bnez" => type_sb_z!(pre::Bne),
        "bltz" => type_sb_z!(pre::Blt),
        "bgez" => type_sb_z!(pre::Bge),
        "bltuz" => type_sb_z!(pre::Bltu),
        "bgeuz" => type_sb_z!(pre::Bgeu),
        "bgtz" => type_sb_reversed_z!(pre::Blt),
        "blez" => type_sb_reversed_z!(pre::Bge),

        // CSR
        "csrw" => csr_small!(CsrRw),
        "csrc" => csr_small!(CsrRc),
        "csrs" => csr_small!(CsrRs),
        "csrwi" => csr_small_imm!(CsrRwi),
        "csrci" => csr_small_imm!(CsrRci),
        "csrsi" => csr_small_imm!(CsrRsi),
        "csrrs" => csr!(CsrRs),
        "csrrw" => csr!(CsrRw),
        "csrrc" => csr!(CsrRc),
        "csrrsi" => csr_imm!(CsrRsi),
        "csrrwi" => csr_imm!(CsrRwi),
        "csrrci" => csr_imm!(CsrRci),
        "csrr" => args_csr_small(s, &regs, &status).map(|(rd, fcsr)| CsrRs(rd, fcsr, 0).into())?,

        // Jumps
        "jal" => parse_jal(s, &regs)?,
        "call" => one_arg(s).map(|(_i, label)| pre::Jal(1, label.to_owned()))?,
        "j" | "tail" | "b" => one_arg(s).map(|(_i, label)| pre::Jal(0, label.to_owned()))?,
        "ret" => Ret.into(),

        "ecall" => Ecall.into(),

        // not quite a `jal`, but the same arguments
        "la" => args_jal(s, &regs).map(|(rd, label)| pre::La(rd, label.to_owned()))?,

        "li" => args_li(s, &regs).map(|(rd, imm)| Li(rd, imm).into())?,
        "lui" => args_li(s, &regs).map(|(rd, imm)| Li(rd, imm << 12).into())?,

        "nop" => Mv(0, 0).into(),

        "fadd.s" => type_r!(float F::Add),
        "fsub.s" => type_r!(float F::Sub),
        "fmul.s" => type_r!(float F::Mul),
        "fdiv.s" => type_r!(float F::Div),
        "feq.s" => type_r!(mixed F::Equ),
        "fle.s" => type_r!(mixed F::Le),
        "flt.s" => type_r!(mixed F::Lt),
        "fmax.s" => type_r!(float F::Max),
        "fmin.s" => type_r!(float F::Min),
        "fsgnj.s" => type_r!(float F::SgnjS),
        "fsgnjn.s" => type_r!(float F::SgnjNS),
        "fsgnjx.s" => type_r!(float F::SgnjXS),
        "fclass.s" => float_two_regs!(F::Class, &regs, &floats),
        "fcvt.s.w" => float_two_regs!(F::CvtSW, &floats, &regs),
        "fcvt.s.wu" => float_two_regs!(F::CvtSWu, &floats, &regs),
        "fcvt.w.s" => float_two_regs!(F::CvtWS, &regs, &floats),
        "fcvt.wu.s" => float_two_regs!(F::CvtWuS, &regs, &floats),
        "fmv.s.x" => float_two_regs!(F::MvSX, &floats, &regs),
        "fmv.x.s" => float_two_regs!(F::MvXS, &regs, &floats),
        "fsqrt.s" => float_two_regs!(F::Sqrt, &floats, &floats),
        "fabs.s" => {
            float_two_regs(s, &floats, &floats).map(|(rd, rs1)| F::SgnjXS(rd, rs1, rs1).into())?
        }
        "fmv.s" => {
            float_two_regs(s, &floats, &floats).map(|(rd, rs1)| F::SgnjS(rd, rs1, rs1).into())?
        }
        "fneg.s" => {
            float_two_regs(s, &floats, &floats).map(|(rd, rs1)| F::SgnjNS(rd, rs1, rs1).into())?
        }
        "flw" => type_s!(float F::Lw),
        "fsw" => type_s!(float F::Sw),

        "uret" => URet.into(),

        dont_know => return Err(Error::InstructionNotFound(dont_know.to_owned())),
    };

    Ok(parsed)
}

/// Parses either `jal rd label` or `jal label`. In the last case, we set `rd = ra`
fn parse_jal<'a>(s: &'a str, regs: &RegMap) -> Result<PreLabelInstruction, Error> {
    use PreLabelInstruction as pre;
    args_jal(s, regs)
        .map(|(rd, label)| pre::Jal(rd, label.to_owned()))
        .or_else(|_| one_arg(s).map(|(_i, label)| pre::Jal(1, label.to_owned())))
        .map_err(|e| e.into())
}

/// Parses a single line of RISC-V code and pushes one or more instructions to the `code` vector
pub(super) fn parse_line(s: &str, regmaps: &FullRegMap, code: &mut Vec<PreLabelInstruction>) -> Result<(), Error> {
    if let Some(instructions) = parse_multi_instruction(s, regmaps) {
        code.extend(instructions);
        return Ok(());
    }

    let i = parse_instruction(s, regmaps)?;
    code.push(i);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Instruction::*;
    use super::PreLabelInstruction as pre;
    use super::*;
    use crate::parser::register_names as reg_names;

    use lazy_static::*;
    lazy_static! {
        static ref FULLREG: FullRegMap =
            { (reg_names::regs(), reg_names::floats(), reg_names::status()) };
    }

    #[test]
    fn test_parse_text() {
        assert_eq!(
            parse_instruction("add s0, s0, s1,,,, ", &FULLREG).map_err(|_| ()),
            Ok(Add(8, 8, 9).into())
        );
        assert_eq!(
            parse_instruction("j label", &FULLREG).map_err(|_| ()),
            Ok(pre::Jal(0, "label".to_owned()).into())
        );
        assert_eq!(
            parse_instruction("bgtz x1 somewhere", &FULLREG).map_err(|_| ()),
            Ok(pre::Blt(0, 1, "somewhere".to_owned()).into())
        );
    }
}
