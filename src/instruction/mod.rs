
/// Floating point instructions.
/// In a separate enum because maybe someday I'll have a cargo feature to disable
/// floating point instructions.
/// Everything here is single precision, no doubles allowed.
#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum FloatInstruction {
    /// rd, rs1, rs2
    Add(u8, u8, u8),
    Sub(u8, u8, u8),
    Mul(u8, u8, u8),
    Div(u8, u8, u8),
    Equ(u8, u8, u8), // Eq was taken
    Le(u8, u8, u8),
    Lt(u8, u8, u8),
    Max(u8, u8, u8),
    Min(u8, u8, u8),
    SgnjS(u8, u8, u8),
    SgnjNS(u8, u8, u8),
    SgnjXS(u8, u8, u8),

    /// rd, rs1
    Class(u8, u8),
    CvtSW(u8, u8),  // fcvt.s.w
    CvtSWu(u8, u8), // fcvt.s.wu
    CvtWS(u8, u8),  // fcvt.w.s
    CvtWuS(u8, u8), // fcvw.wu.s
    MvSX(u8, u8),   // fmv.s.x
    MvXS(u8, u8),   // fmv.x.s
    Sqrt(u8, u8),

    Lw(u8, u32, u8),
    Sw(u8, u32, u8),
}

/// Giant enum that represents a single RISC-V instruction and its arguments
#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Instruction {
    // Type R
    /// rd, rs1, rs2
    Add(u8, u8, u8),
    Sub(u8, u8, u8),
    Sll(u8, u8, u8),
    Slt(u8, u8, u8),
    Sltu(u8, u8, u8),
    Xor(u8, u8, u8),
    Srl(u8, u8, u8),
    Sra(u8, u8, u8),
    Or(u8, u8, u8),
    And(u8, u8, u8),
    Mul(u8, u8, u8), // TODO: mulh, mulhsu, mulhu
    Div(u8, u8, u8),
    Divu(u8, u8, u8),
    Rem(u8, u8, u8),
    Remu(u8, u8, u8),

    // Type I
    Ecall,
    /// rd, imm, rs1
    Lb(u8, u32, u8),
    Lh(u8, u32, u8),
    Lw(u8, u32, u8),
    Lbu(u8, u32, u8),
    Lhu(u8, u32, u8),
    /// rd, rs1, imm
    Addi(u8, u8, u32),
    Slti(u8, u8, u32),
    Sltiu(u8, u8, u32),
    Slli(u8, u8, u32),
    Srli(u8, u8, u32),
    Srai(u8, u8, u32),
    Ori(u8, u8, u32),
    Andi(u8, u8, u32),
    Xori(u8, u8, u32),

    // Type S
    /// rs2, imm, rs1
    Sb(u8, u32, u8),
    Sh(u8, u32, u8),
    Sw(u8, u32, u8),

    // Type SB + jumps
    /// rs1, rs2, label
    Beq(u8, u8, usize),
    Bne(u8, u8, usize),
    Blt(u8, u8, usize),
    Bge(u8, u8, usize),
    Bltu(u8, u8, usize),
    Bgeu(u8, u8, usize),
    /// rd, rs1, imm
    Jalr(u8, u8, u32),
    /// rd, label
    Jal(u8, usize),

    // CSR
    /// rd, fcsr, rs1
    CsrRw(u8, u8, u8),
    CsrRs(u8, u8, u8),
    CsrRc(u8, u8, u8),
    /// rd, fcsr, imm
    CsrRwi(u8, u8, u32),
    CsrRsi(u8, u8, u32),
    CsrRci(u8, u8, u32),

    // Floating point
    Float(FloatInstruction),

    // Some pseudoinstructions
    /// rd, imm
    Li(u8, u32),
    /// rd, rs1
    Mv(u8, u8),

    Ret,
    URet,
}

