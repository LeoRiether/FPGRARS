//!
//! Parses RISC-V code into code and data parts, so it can be used in the simulator module.
//! We use a lot of mnemonics here, I'll try to link to a cheatsheet here later.
//!

use combine::*;
use radix_trie::Trie;
use std::fmt;

mod register_names;
use register_names as reg_names;

mod preprocessor;
pub use preprocessor::*;

mod util;
pub use util::*;


/// Giant enum that represents a single RISC-V instruction and its arguments
#[allow(dead_code)] // please, cargo, no more warnings
#[derive(Debug)]
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
    Lb(u8, i32, u8),
    Lh(u8, i32, u8),
    Lw(u8, i32, u8),
    Lbu(u8, i32, u8),
    Lhu(u8, i32, u8),
    Addi(u8, u8, i32),
    /// rd, rs1, imm
    Slti(u8, u8, i32),
    Sltiu(u8, u8, u32),
    Slli(u8, u8, i32),
    Srli(u8, u8, i32),
    Srai(u8, u8, i32),
    Ori(u8, u8, u32),
    Andi(u8, u8, u32),
    Xori(u8, u8, u32),

    // Type S
    /// rs2, imm, rs1
    Sb(u8, i32, u8),
    Sh(u8, i32, u8),
    Sw(u8, i32, u8),

    // Type SB + jumps
    /// rs1, rs2, label
    Beq(u8, u8, usize),
    Bne(u8, u8, usize),
    Blt(u8, u8, usize),
    Bge(u8, u8, usize),
    Bltu(u8, u8, usize),
    Bgeu(u8, u8, usize),
    /// rd, rs1, imm
    Jalr(u8, u8, i32),
    /// rd, label
    Jal(u8, usize),

    // Some pseudoinstructions
    /// rd, imm
    Li(u8, i32),
    /// rd, rs1
    Mv(u8, u8),
    /// rd, label
    La(u8, usize),

    Ret,
}

/// Also giant enum that represents a single RISC-V instruction, but we save
/// labels as strings because it might not have parsed it yet (for example,
/// consider a jump instruction that jumps to a label in the next line).
///
/// We process the labels stored after the entire file has been parsed.
enum PreLabelInstruction {
    Beq(u8, u8, String),
    Bne(u8, u8, String),
    Blt(u8, u8, String),
    Bge(u8, u8, String),
    Bltu(u8, u8, String),
    Bgeu(u8, u8, String),
    Jal(u8, String),
    La(u8, String),
    Other(Instruction),
}

/// Represents a successful parser result. This is the same format the simulator
/// will use to execute the instructions
pub struct Parsed {
    pub code: Vec<Instruction>,
    pub data: Vec<u8>,
}


pub type ParseResult = Result<Parsed, Error>;

/// The "current" parser directive
enum Directive {
    Text,
    Data,
}

pub trait LineParser {
    fn parse_riscv(self, data_segment_size: usize) -> ParseResult;
}

impl<I: Iterator<Item = String>> LineParser for I
{
    fn parse_riscv(self, data_segment_size: usize) -> ParseResult {
        let regmap = reg_names::regs();
        let floatmap = reg_names::floats();
        let statusmap = reg_names::status();
        let labels = Trie::<String, usize>::new();

        let mut code = Vec::new();
        let mut data = Vec::with_capacity(data_segment_size);

        for line in self {
            println!("> {}", line);
        }

        let code: Result<Vec<Instruction>, Error> = code
            .into_iter()
            .map(|i| unlabel_instruction(i, &labels))
            .collect();
        let mut code = code?;

        code.extend(vec![
            Instruction::Li(17, 10), // li a7 10
            Instruction::Ecall,
        ]);

        data.resize(data_segment_size, 0);
        Ok(Parsed { code, data })
    }
}

/// Transforms a PreLabelInstruction into a normal Instruction by "commiting" the labels
/// into positions in the code. For example, Jal(0, "Label") maps to Jal(0, labels_trie.get("Label"))
fn unlabel_instruction(
    instruction: PreLabelInstruction,
    labels: &Trie<String, usize>,
) -> Result<Instruction, Error> {
    use Instruction::*;
    use PreLabelInstruction as p;

    // TODO: refactor this, maybe
    match instruction {
        p::Jal(rd, label) => labels
            .get(&label)
            .map(|&pos| Jal(rd, pos))
            .ok_or(Error::LabelNotFound(label)),
        p::Beq(rs1, rs2, label) => labels
            .get(&label)
            .map(|&pos| Beq(rs1, rs2, pos))
            .ok_or(Error::LabelNotFound(label)),
        p::Bne(rs1, rs2, label) => labels
            .get(&label)
            .map(|&pos| Bne(rs1, rs2, pos))
            .ok_or(Error::LabelNotFound(label)),
        p::Bge(rs1, rs2, label) => labels
            .get(&label)
            .map(|&pos| Bge(rs1, rs2, pos))
            .ok_or(Error::LabelNotFound(label)),
        p::Blt(rs1, rs2, label) => labels
            .get(&label)
            .map(|&pos| Blt(rs1, rs2, pos))
            .ok_or(Error::LabelNotFound(label)),
        p::Bltu(rs1, rs2, label) => labels
            .get(&label)
            .map(|&pos| Bltu(rs1, rs2, pos))
            .ok_or(Error::LabelNotFound(label)),
        p::Bgeu(rs1, rs2, label) => labels
            .get(&label)
            .map(|&pos| Bgeu(rs1, rs2, pos))
            .ok_or(Error::LabelNotFound(label)),
        p::La(rd, label) => labels
            .get(&label)
            .map(|&pos| La(rd, pos))
            .ok_or(Error::LabelNotFound(label)),
        p::Other(instruction) => Ok(instruction),
    }
}
