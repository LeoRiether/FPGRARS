//!
//! Parses RISC-V code into code and data parts, so it can be used in the simulator module.
//! We use a lot of mnemonics here, I'll try to link to a cheatsheet here later.
//!

use radix_trie::Trie;
use byteorder::{ByteOrder, LittleEndian};

pub mod register_names;
use register_names::{self as reg_names, FullRegMap};

pub mod combinators;

mod preprocessor;
pub use preprocessor::*;

mod util;
pub use util::*;

mod data;
mod text;

/// Floating point instructions.
/// In a separate enum because maybe someday I'll have a cargo feature to disable
/// floating point instructions.
/// Everything here is single precision, no doubles allowed.
#[derive(Debug, PartialEq, Eq)]
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

/// Also giant enum that represents a single RISC-V instruction, but we save
/// labels as strings because it might not have parsed it yet (for example,
/// consider a jump instruction that jumps to a label in the next line).
///
/// We process the labels stored after the entire file has been parsed.
#[derive(Debug, PartialEq, Eq)]
enum PreLabelInstruction {
    Beq(u8, u8, String),
    Bne(u8, u8, String),
    Blt(u8, u8, String),
    Bge(u8, u8, String),
    Bltu(u8, u8, String),
    Bgeu(u8, u8, String),
    Jal(u8, String),

    /// Gets mapped to an Instruction::Li(rd, position) after unlabeling
    La(u8, String),

    Other(Instruction),
}

impl From<Instruction> for PreLabelInstruction {
    fn from(i: Instruction) -> PreLabelInstruction {
        PreLabelInstruction::Other(i)
    }
}

impl From<FloatInstruction> for PreLabelInstruction {
    fn from(i: FloatInstruction) -> PreLabelInstruction {
        PreLabelInstruction::Other(Instruction::Float(i))
    }
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

pub trait RISCVParser {
    /// Parses an iterator of preprocessed lines and returns the instructions and
    /// the data it parsed. Remember to preprocess the iterator before calling this,
    /// as `parse_riscv` does not understand macros and includes.
    /// ```
    /// parse::file_lines("riscv.s".to_owned())?
    ///     .parse_includes()
    ///     .parse_macros()
    ///     .parse_riscv(DATA_SIZE)?;
    /// ```
    ///
    /// The `data_segment_size` parameter is the final size of the data segment, in bytes.
    fn parse_riscv(self, data_segment_size: usize) -> ParseResult;
}

impl<I: Iterator<Item = String>> RISCVParser for I {
    fn parse_riscv(self, data_segment_size: usize) -> ParseResult {
        use combinators::*;

        let regmaps: FullRegMap = (reg_names::regs(), reg_names::floats(), reg_names::status());
        let mut labels = Trie::<String, usize>::new();

        let mut directive = Directive::Text;
        let mut code = Vec::new();

        let mut data = Vec::with_capacity(data_segment_size);
        let mut current_data_type = data::Type::default();
        let mut data_labels: Vec<data::Label> = Vec::new();

        for line in self {
            let full_line = &line;

            let line = match parse_label(&line) {
                Ok((rest, label)) => {
                    let label_pos = match directive {
                        Directive::Text => code.len() * 4,
                        Directive::Data => data.len(),
                    };
                    labels.insert(label.to_owned(), label_pos);
                    rest
                }
                Err(_) => &line,
            };

            let (line, _) = separator0(line)?;
            if line.is_empty() {
                continue;
            }

            // Identify directives
            // This accepts stuff like ".textSOMETHING" or ".database", but RARS accepts it too
            // Gotta be consistent! ¯\_(ツ)_/¯
            if line.starts_with(".data") {
                directive = Directive::Data;
                continue;
            } else if line.starts_with(".text") {
                directive = Directive::Text;
                continue;
            }

            let res = match directive {
                Directive::Text => text::parse_line(line, &regmaps, &mut code),
                Directive::Data => {
                    data::parse_line(line, &mut data, &mut data_labels, &mut current_data_type)
                }
            };

            res.wrap_meta(full_line)?;
        }

        unlabel_data(data_labels, &mut data, &labels)?;

        let code: Result<Vec<Instruction>, Error> = code
            .into_iter()
            .map(|i| unlabel_instruction(i, &labels))
            .collect();
        let mut code = code?;

        // If the program ever drops off bottom, we make an "exit" ecall and terminate execution
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

    macro_rules! unlabel {
        ($inst:ident, $rd:ident, $label:ident) => {
            labels
                .get(&$label)
                .map(|&pos| $inst($rd, pos))
                .ok_or(Error::LabelNotFound($label))
        };
        ($inst:ident, $rs1:ident, $rs2:ident, $label:ident) => {
            labels
                .get(&$label)
                .map(|&pos| $inst($rs1, $rs2, pos))
                .ok_or(Error::LabelNotFound($label))
        };
    }

    match instruction {
        p::Jal(rd, label) => unlabel!(Jal, rd, label),
        p::Beq(rs1, rs2, label) => unlabel!(Beq, rs1, rs2, label),
        p::Bne(rs1, rs2, label) => unlabel!(Bne, rs1, rs2, label),
        p::Bge(rs1, rs2, label) => unlabel!(Bge, rs1, rs2, label),
        p::Blt(rs1, rs2, label) => unlabel!(Blt, rs1, rs2, label),
        p::Bltu(rs1, rs2, label) => unlabel!(Bltu, rs1, rs2, label),
        p::Bgeu(rs1, rs2, label) => unlabel!(Bgeu, rs1, rs2, label),

        p::La(rd, label) => labels
            .get(&label)
            .map(|&pos| Li(rd, pos as u32))
            .ok_or(Error::LabelNotFound(label)),

        p::Other(instruction) => Ok(instruction),
    }
}

/// Replaces all positions in the `.data` that had labels with their
/// actual values
fn unlabel_data(data_labels: Vec<data::Label>, data: &mut Vec<u8>, labels: &Trie<String, usize>) -> Result<(), Error> {
    for dl in data_labels {
        let data::Label{ pos, dtype, label } = dl;

        let value = match labels.get(&label) {
            Some(x) => *x,
            None => return Err(Error::LabelNotFound(label)),
        };

        use data::Type::*;
        match dtype {
            Byte => { data[pos] = value as u8; }
            Half => LittleEndian::write_u16(&mut data[pos..], value as u16),
            Word => LittleEndian::write_u32(&mut data[pos..], value as u32),
            _ => unreachable!("label can only be parsed in .byte, .half or .word"),
        }
    }

    Ok(())
}