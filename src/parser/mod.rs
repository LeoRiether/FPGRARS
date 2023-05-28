//!
//! Parses RISC-V code into code and data parts, so it can be used in the simulator module.
//! We use a lot of mnemonics here, I'll try to link to a cheatsheet here later.
//!

pub mod combinators;
mod data;
pub mod error;
pub mod lexer;
mod preprocessor;
pub mod register_names;
mod text;
pub mod token;
mod util;

use crate::instruction::{FloatInstruction, Instruction, PreLabelInstruction};
use byteorder::{ByteOrder, LittleEndian};
use error::{Error, ParserError};
use hashbrown::HashMap;
pub use preprocessor::*;
use register_names::{self as reg_names, FullRegMap};
pub use util::*;

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

impl<I: Iterator<Item = Result<String, ParserError>>> RISCVParser for I {
    fn parse_riscv(self, data_segment_size: usize) -> ParseResult {
        use combinators::*;

        let regmaps: FullRegMap = (reg_names::regs(), reg_names::floats(), reg_names::status());
        let mut labels = HashMap::<String, usize>::new();

        let mut directive = Directive::Text;
        let mut code = Vec::new();

        let mut data = Vec::with_capacity(data_segment_size);
        let mut current_data_type = data::Type::default();
        let mut data_labels: Vec<data::Label> = Vec::new();

        for line in self {
            let line = line?;

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

            let (line, _) = separator0(line).map_err(ParserError::from)?;
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
    labels: &HashMap<String, usize>,
) -> Result<Instruction, ParserError> {
    use Instruction::*;
    use PreLabelInstruction as p;

    macro_rules! unlabel {
        ($inst:ident, $rd:ident, $label:ident) => {
            labels
                .get(&$label)
                .map(|&pos| $inst($rd, pos))
                .ok_or(ParserError::LabelNotFound($label))
        };
        ($inst:ident, $rs1:ident, $rs2:ident, $label:ident) => {
            labels
                .get(&$label)
                .map(|&pos| $inst($rs1, $rs2, pos))
                .ok_or(ParserError::LabelNotFound($label))
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
            .ok_or(ParserError::LabelNotFound(label)),

        p::Other(instruction) => Ok(instruction),
    }
}

/// Replaces all positions in the `.data` that had labels with their
/// actual values
fn unlabel_data(
    data_labels: Vec<data::Label>,
    data: &mut [u8],
    labels: &HashMap<String, usize>,
) -> Result<(), Error> {
    for dl in data_labels {
        let data::Label { pos, dtype, label } = dl;

        let value = match labels.get(&label) {
            Some(x) => *x,
            None => return Err(ParserError::LabelNotFound(label).into()),
        };

        use data::Type::*;
        match dtype {
            Byte => {
                data[pos] = value as u8;
            }
            Half => LittleEndian::write_u16(&mut data[pos..], value as u16),
            Word => LittleEndian::write_u32(&mut data[pos..], value as u32),
            _ => unreachable!("label can only be parsed in .byte, .half or .word"),
        }
    }

    Ok(())
}
