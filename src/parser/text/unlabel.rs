use std::mem::replace;

use crate::instruction::{FloatInstruction, Instruction};

pub fn unlabel(code: &mut [Instruction], i: usize, label: usize) {
    use Instruction::*;

    let label32 = label as u32;
    let instr = replace(&mut code[i], Add(0, 0, 0));

    code[i] = match instr {
        Lb(rd, _, rs1) => Lb(rd, label32, rs1),
        Lh(rd, _, rs1) => Lh(rd, label32, rs1),
        Lw(rd, _, rs1) => Lw(rd, label32, rs1),
        Lbu(rd, _, rs1) => Lbu(rd, label32, rs1),
        Lhu(rd, _, rs1) => Lhu(rd, label32, rs1),
        Addi(rd, rs1, _) => Addi(rd, rs1, label32),
        Slti(rd, rs1, _) => Slti(rd, rs1, label32),
        Sltiu(rd, rs1, _) => Sltiu(rd, rs1, label32),
        Slli(rd, rs1, _) => Slli(rd, rs1, label32),
        Srli(rd, rs1, _) => Srli(rd, rs1, label32),
        Srai(rd, rs1, _) => Srai(rd, rs1, label32),
        Ori(rd, rs1, _) => Ori(rd, rs1, label32),
        Andi(rd, rs1, _) => Andi(rd, rs1, label32),
        Xori(rd, rs1, _) => Xori(rd, rs1, label32),
        Sb(rs2, _, rs1) => Sb(rs2, label32, rs1),
        Sh(rs2, _, rs1) => Sh(rs2, label32, rs1),
        Sw(rs2, _, rs1) => Sw(rs2, label32, rs1),
        Beq(rs1, rs2, _) => Beq(rs1, rs2, label),
        Bne(rs1, rs2, _) => Bne(rs1, rs2, label),
        Blt(rs1, rs2, _) => Blt(rs1, rs2, label),
        Bge(rs1, rs2, _) => Bge(rs1, rs2, label),
        Bltu(rs1, rs2, _) => Bltu(rs1, rs2, label),
        Bgeu(rs1, rs2, _) => Bgeu(rs1, rs2, label),
        Jalr(rd, rs1, _) => Jalr(rd, rs1, label32),
        Jal(rd, _) => Jal(rd, label),
        CsrRwi(rd, fcsr, _) => CsrRwi(rd, fcsr, label32),
        CsrRsi(rd, fcsr, _) => CsrRsi(rd, fcsr, label32),
        CsrRci(rd, fcsr, _) => CsrRci(rd, fcsr, label32),
        Float(f) => Float(unlabel_float(f, label)),
        Li(rd, _) => Li(rd, label32),
        _ => panic!("FPGRARS tried to unlabel an instruction that doesn't have a label! {instr:?}"),
    };
}

fn unlabel_float(f: FloatInstruction, label: usize) -> FloatInstruction {
    use FloatInstruction::*;
    let label32 = label as u32;
    match f {
        Lw(rd, _, rs1) => Lw(rd, label32, rs1),
        Sw(rs2, _, rs1) => Sw(rs2, label32, rs1),
        _ => panic!("FPGRARS tried to unlabel an instruction that doesn't have a label! {f:?}"),
    }
}
