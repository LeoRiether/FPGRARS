pub mod bitops;
pub mod constants;
pub mod instruction;

pub use instruction::Instruction;

use crate::parser;
use constants::*;

macro_rules! compile_inner {
    ($i:ident) => {};
    ($i:ident, ) => {};
    ($i:ident, opcode: $val:expr; $($props:tt)*) => {
        $i.set_opcode($val as u32);
        compile_inner!($i, $($props)*);
    };
    ($i:ident, rd: $val:expr; $($props:tt)*) => {
        $i.set_rd($val as u32);
        compile_inner!($i, $($props)*);
    };
    ($i:ident, funct3: $val:expr; $($props:tt)*) => {
        $i.set_funct3($val as u32);
        compile_inner!($i, $($props)*);
    };
    ($i:ident, funct7: $val:expr; $($props:tt)*) => {
        $i.set_funct7($val as u32);
        compile_inner!($i, $($props)*);
    };
    ($i:ident, rs1: $val:expr; $($props:tt)*) => {
        $i.set_rs1($val as u32);
        compile_inner!($i, $($props)*);
    };
    ($i:ident, rs2: $val:expr; $($props:tt)*) => {
        $i.set_rs2($val as u32);
        compile_inner!($i, $($props)*);
    };
    ($i:ident, imm_i: $val:expr; $($props:tt)*) => {
        $i.set_imm_i($val as i32);
        compile_inner!($i, $($props)*);
    };
    ($i:ident, imm_s: $val:expr; $($props:tt)*) => {
        $i.set_imm_s($val as i32);
        compile_inner!($i, $($props)*);
    };
    ($i:ident, imm_b: $val:expr; $($props:tt)*) => {
        $i.set_imm_b($val as i32);
        compile_inner!($i, $($props)*);
    };
    ($i:ident, imm_j: $val:expr; $($props:tt)*) => {
        $i.set_imm_j($val as i32);
        compile_inner!($i, $($props)*);
    };
}

macro_rules! opcode_shorthand {
    (R) => {
        OPCODE_TYPE_R
    };
    (I_IMM) => {
        OPCODE_TYPE_I_IMM
    };
    (I_LOAD) => {
        OPCODE_TYPE_I_LOAD
    };
    (I_JALR) => {
        OPCODE_TYPE_I_JALR
    };
    (I_SYS) => {
        OPCODE_TYPE_I_SYSTEM
    };
    (S) => {
        OPCODE_TYPE_S
    };
    (B) => {
        OPCODE_TYPE_B
    };
    (J) => {
        OPCODE_TYPE_J
    };
}

macro_rules! compile {
    ($opcode:ident; $($props:tt)*) => {
        compile!{ opcode: opcode_shorthand!($opcode); $($props)* }
    };
    ($($props:tt)*) => {
        {
            let mut instruction = Instruction(0);
            compile_inner!(instruction, $($props)*);
            instruction
        }
    };
}

impl Instruction {
    pub fn from_parsed(instruction: parser::Instruction, pc: usize) -> Self {
        use parser::Instruction::*;

        // TODO: disallow unused variables when this is done
        #[allow(unused_variables)]
        match instruction {
            Add(rd, rs1, rs2) => {
                compile! { R; funct3: add::F3; funct7: add::F7; rd: rd; rs1: rs1; rs2: rs2; }
            }
            Sub(rd, rs1, rs2) => {
                compile! { R; funct3: sub::F3; funct7: sub::F7; rd: rd; rs1: rs1; rs2: rs2; }
            }
            Sll(rd, rs1, rs2) => {
                compile! { R; funct3: sll::F3; funct7: sll::F7; rd: rd; rs1: rs1; rs2: rs2; }
            }
            Slt(rd, rs1, rs2) => {
                compile! { R; funct3: slt::F3; funct7: slt::F7; rd: rd; rs1: rs1; rs2: rs2; }
            }
            Sltu(rd, rs1, rs2) => {
                compile! { R; funct3: sltu::F3; funct7: sltu::F7; rd: rd; rs1: rs1; rs2: rs2; }
            }
            Xor(rd, rs1, rs2) => {
                compile! { R; funct3: xor::F3; funct7: xor::F7; rd: rd; rs1: rs1; rs2: rs2; }
            }
            Srl(rd, rs1, rs2) => {
                compile! { R; funct3: srl::F3; funct7: srl::F7; rd: rd; rs1: rs1; rs2: rs2; }
            }
            Sra(rd, rs1, rs2) => {
                compile! { R; funct3: sra::F3; funct7: sra::F7; rd: rd; rs1: rs1; rs2: rs2; }
            }
            Or(rd, rs1, rs2) => {
                compile! { R; funct3: or::F3; funct7: or::F7; rd: rd; rs1: rs1; rs2: rs2; }
            }
            And(rd, rs1, rs2) => {
                compile! { R; funct3: and::F3; funct7: and::F7; rd: rd; rs1: rs1; rs2: rs2; }
            }
            Mul(rd, rs1, rs2) => {
                compile! { R; funct3: mul::F3; funct7: mul::F7; rd: rd; rs1: rs1; rs2: rs2; }
            }
            Div(rd, rs1, rs2) => {
                compile! { R; funct3: div::F3; funct7: div::F7; rd: rd; rs1: rs1; rs2: rs2; }
            }
            Divu(rd, rs1, rs2) => {
                compile! { R; funct3: divu::F3; funct7: divu::F7; rd: rd; rs1: rs1; rs2: rs2; }
            }
            Rem(rd, rs1, rs2) => {
                compile! { R; funct3: rem::F3; funct7: rem::F7; rd: rd; rs1: rs1; rs2: rs2; }
            }
            Remu(rd, rs1, rs2) => {
                compile! { R; funct3: remu::F3; funct7: remu::F7; rd: rd; rs1: rs1; rs2: rs2; }
            }
            Ecall => compile! { I_SYS; funct3: ecall::F3; funct7: ecall::F7; },
            Lb(rd, imm, rs1) => compile! { I_LOAD; funct3: lb::F3; rd: rd; rs1: rs1; imm_i: imm; },
            Lh(rd, imm, rs1) => compile! { I_LOAD; funct3: lh::F3; rd: rd; rs1: rs1; imm_i: imm; },
            Lw(rd, imm, rs1) => compile! { I_LOAD; funct3: lw::F3; rd: rd; rs1: rs1; imm_i: imm; },
            Lbu(rd, imm, rs1) => {
                compile! { I_LOAD; funct3: lbu::F3; rd: rd; rs1: rs1; imm_i: imm; }
            }
            Lhu(rd, imm, rs1) => {
                compile! { I_LOAD; funct3: lhu::F3; rd: rd; rs1: rs1; imm_i: imm; }
            }
            Addi(rd, rs1, imm) => {
                compile! { I_IMM; funct3: addi::F3; rd: rd; rs1: rs1; imm_i: imm; }
            }
            Slti(rd, rs1, imm) => {
                compile! { I_IMM; funct3: slti::F3; rd: rd; rs1: rs1; imm_i: imm; }
            }
            Sltiu(rd, rs1, imm) => {
                compile! { I_IMM; funct3: sltiu::F3; rd: rd; rs1: rs1; imm_i: imm; }
            }
            Slli(rd, rs1, imm) => {
                compile! { I_IMM; funct3: slli::F3; rd: rd; rs1: rs1; imm_i: imm; }
            }
            Srli(rd, rs1, imm) => {
                compile! { I_IMM; funct3: srli::F3; funct7: srli::F7; rd: rd; rs1: rs1; imm_i: imm; }
            }
            Srai(rd, rs1, imm) => {
                compile! { I_IMM; funct3: srai::F3; funct7: srai::F7; rd: rd; rs1: rs1; imm_i: imm; }
            }
            Ori(rd, rs1, imm) => compile! { I_IMM; funct3: ori::F3; rd: rd; rs1: rs1; imm_i: imm; },
            Andi(rd, rs1, imm) => {
                compile! { I_IMM; funct3: andi::F3; rd: rd; rs1: rs1; imm_i: imm; }
            }
            Xori(rd, rs1, imm) => {
                compile! { I_IMM; funct3: xori::F3; rd: rd; rs1: rs1; imm_i: imm; }
            }
            Sb(rs2, imm, rs1) => compile! { S; funct3: sb::F3; rs1: rs1; rs2: rs2; imm_s: imm; },
            Sh(rs2, imm, rs1) => compile! { S; funct3: sh::F3; rs1: rs1; rs2: rs2; imm_s: imm; },
            Sw(rs2, imm, rs1) => compile! { S; funct3: sw::F3; rs1: rs1; rs2: rs2; imm_s: imm; },
            Beq(rs1, rs2, label) => {
                compile! { B; funct3: beq::F3; rs1: rs1; rs2: rs2; imm_b: (label as isize - pc as isize); }
            }
            Bne(rs1, rs2, label) => {
                compile! { B; funct3: bne::F3; rs1: rs1; rs2: rs2; imm_b: (label as isize - pc as isize); }
            }
            Blt(rs1, rs2, label) => {
                compile! { B; funct3: blt::F3; rs1: rs1; rs2: rs2; imm_b: (label as isize - pc as isize); }
            }
            Bge(rs1, rs2, label) => {
                compile! { B; funct3: bge::F3; rs1: rs1; rs2: rs2; imm_b: (label as isize - pc as isize); }
            }
            Bltu(rs1, rs2, label) => {
                compile! { B; funct3: bltu::F3; rs1: rs1; rs2: rs2; imm_b: (label as isize - pc as isize); }
            }
            Bgeu(rs1, rs2, label) => {
                compile! { B; funct3: bgeu::F3; rs1: rs1; rs2: rs2; imm_b: (label as isize - pc as isize); }
            }
            Jalr(rd, rs1, imm) => {
                compile! { I_JALR; funct3: jalr::F3; rd: rd; rs1: rs1; imm_i: imm; }
            }
            Jal(rd, label) => compile! { J; rd: rd; imm_j: (label as isize - pc as isize); },
            CsrRw(rd, fcsr, rs1) => {
                compile! { I_SYS; funct3: csrrw::F3; rd: rd; rs1: rs1; imm_i: fcsr; }
            }
            CsrRs(rd, fcsr, rs1) => {
                compile! { I_SYS; funct3: csrrs::F3; rd: rd; rs1: rs1; imm_i: fcsr; }
            }
            CsrRc(rd, fcsr, rs1) => {
                compile! { I_SYS; funct3: csrrc::F3; rd: rd; rs1: rs1; imm_i: fcsr; }
            }
            CsrRwi(rd, fcsr, imm) => {
                compile! { I_SYS; funct3: csrrwi::F3; rd: rd; rs1: imm; imm_i: fcsr; }
            }
            CsrRsi(rd, fcsr, imm) => {
                compile! { I_SYS; funct3: csrrsi::F3; rd: rd; rs1: imm; imm_i: fcsr; }
            }
            CsrRci(rd, fcsr, imm) => {
                compile! { I_SYS; funct3: csrrci::F3; rd: rd; rs1: imm; imm_i: fcsr; }
            }
            Float(_) => Instruction(0),
            Li(_, _) => Instruction(0),
            URet => Instruction(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::parser::Instruction as ParserInstr;

    #[test]
    fn test_jal() {
        assert_eq!(
            Instruction::from_parsed(ParserInstr::Jal(5, 3), 0),
            Instruction(0x00c002ef)
        );
    }
}
