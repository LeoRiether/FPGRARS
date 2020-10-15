//!
//! Parses RISC-V code into code and data parts, so it can be used in the simulator module.
//! We use a lot of mnemonics here, I'll try to link to a cheatsheet here later.
//!

use combine::*;
use radix_trie::Trie;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

mod register_names;
use register_names as reg_names;

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
    Addi(u8, u8, i32), /// rd, rs1, imm
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
    Other(Instruction),
}

/// Represents a successful parser result. This is the same format the simulator
/// will use to execute the instructions
pub struct Parsed {
    pub code: Vec<Instruction>,
    pub data: Vec<u8>,
}

/// Represents any kind of error the parser may find
#[derive(Debug)]
pub enum Error {
    /// Not the parser's fault, some std::io went wrong
    IO(io::Error),
    LabelNotFound(String),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IO(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self) // muahahahaha
    }
}

impl std::error::Error for Error {}

pub type ParseResult = Result<Parsed, Error>;

/// The "current" parser directive
enum Directive {
    Text,
    Data,
}

/// The "current" state of the parser
struct Context {
    regmap: reg_names::RegMap,
    floatmap: reg_names::RegMap,
    statusmap: reg_names::RegMap,

    directive: Directive,
    code: Vec<PreLabelInstruction>,
    data: Vec<u8>,
    labels: Trie<String, usize>,
}

impl Context {
    fn new() -> Self {
        Self {
            regmap: reg_names::regs(),
            floatmap: reg_names::floats(),
            statusmap: reg_names::status(),

            directive: Directive::Text,
            code: Vec::new(),
            data: Vec::new(), // TODO: Vec::with_capacity(final data size)
            labels: Trie::new(),
        }
    }
}

fn parse_file_with_context(path: impl AsRef<Path>, ctx: &mut Context) -> Result<(), Error> {
    let reader = File::open(path).map(BufReader::new)?;
    let lines = reader.lines().map(|x| x.unwrap());
    parse_with_context(lines, ctx)
}

/// Parses some lines with a given mutable context, instead of returning a context.
/// This is done to make the .include "file" directive easier to implement, as we can have
/// just one context shared among recursive parse calls
fn parse_with_context<S, T>(lines: T, ctx: &mut Context) -> Result<(), Error>
where
    S: AsRef<str>,
    T: Iterator<Item = S>,
{
    for (line_number, line) in lines.enumerate() {
        if line.as_ref().starts_with(".include") {
            // parse_file_with_context(path, ctx)?;
        }
    }
    Ok(())
}

/// Parses an iterator of lines. Generally only called by [parse_file](fn.parse_file)
pub fn parse_lines<S, T>(lines: T) -> ParseResult
where
    S: AsRef<str>,
    T: Iterator<Item = S>,
{
    let mut ctx = Context::new();
    parse_with_context(lines, &mut ctx)?;

    let labels = &ctx.labels;
    let code: Result<Vec<Instruction>, Error> = ctx
        .code
        .into_iter()
        .map(|i| unlabel_instruction(i, labels))
        .collect();
    let mut code = code?;

    //* Sample code, just to test things out
    use Instruction::*;
    code.extend(vec![
        Li(8, 0xff000000u32 as i32),
        Li(9, 76800),
        Add(9, 9, 8),
        Bge(8, 9, 8 * 4),
        Li(5, 0xf3),
        Sb(5, 0, 8),
        Addi(8, 8, 1),
        Jal(0, 3 * 4),
        Jal(0, 8 * 4), // main.stall
    ]);

    code.extend(vec![
        Instruction::Li(17, 10), // li a7 10
        Instruction::Ecall,
    ]);

    let data = ctx.data;
    Ok(Parsed { code, data })
}

pub fn parse_file(path: impl AsRef<Path>) -> ParseResult {
    let reader = File::open(path).map(BufReader::new)?;

    // I feel a bit bad about this unwrap, but, like, really? it's going to fail now?
    let parsed = parse_lines(reader.lines().map(|x| x.unwrap()))?;
    Ok(parsed)
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
        p::Other(instruction) => Ok(instruction),
    }
}
