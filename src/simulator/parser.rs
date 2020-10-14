///
/// Parses RISC-V code into code and data parts, so it can be used in the simulator module
///
use combine::*;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::fmt;
use radix_trie::Trie;

/// Giant enum that represents a single RISC-V instruction and its arguments
pub enum Instruction {
    // Type I
    Lb(usize, i32, usize), // rd, imm, rs1
    Lh(usize, i32, usize),
    Lw(usize, i32, usize),
    Lbu(usize, i32, usize),
    Addi(usize, usize, i32), // rd, rs1, imm
    Slli(usize, usize, i32),
    Srai(usize, usize, i32),
}

/// Also giant enum that represents a single RISC-V instruction, but we save
/// labels as strings because it might not have parsed it yet (for example,
/// consider a jump instruction that jumps to a label in the next line).
///
/// We process the labels stored after the entire file has been parsed.
enum PreLabelInstruction {
    Jal(usize, String),
    Beq(usize, usize, String),
    Bge(usize, usize, String),

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

impl std::error::Error for Error { }

pub type ParseResult = Result<Parsed, Error>;

/// The parser state, as it parses each line
enum State {
    Text,
    Data,
}

/// Parses an iterator of lines. Generally only called by [parse_file](fn.parse_file)
pub fn parse_lines<S, T>(lines: T) -> ParseResult
where
    S: AsRef<str>,
    T: Iterator<Item = S>,
{
    let mut state = State::Text;
    let mut code = Vec::<Instruction>::new();
    let mut data = Vec::<u8>::new();
    let mut labels = Trie::<String, usize>::new();

    for (line_number, line) in lines.enumerate() {
        // if we parse a directive, change state, otherwise parse according to state
    }
    unimplemented!()
}

pub fn parse_file(path: impl AsRef<Path>) -> ParseResult {
    let reader = File::open(path).map(BufReader::new)?;

    // I feel a bit bad about this unwrap, but, like, really? it's going to fail now?
    let parsed = parse_lines(reader.lines().map(|x| x.unwrap()))?;
    Ok(parsed)
}
