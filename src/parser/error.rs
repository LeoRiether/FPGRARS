use std::io;
use thiserror::Error;

/// Represents any kind of error the parser may find
#[derive(Debug, Error)]
pub enum ParserError {
    /// Not the parser's fault, some std::io went wrong
    #[error("I/O Error: {0}")]
    IO(#[from] io::Error),

    #[error("Label '{0}' not found")]
    LabelNotFound(String),
    #[error("Parser error '{0}': {1:?}")]
    Nom(String, nom::error::ErrorKind), // I'm feeling lazy
    #[error("Expected a register name, but found '{0}'")]
    RegisterNotFound(String),
    #[error("The instruction '{0}' is either invalid or not implemented in FPGRARS")]
    InstructionNotFound(String),

    #[error("Error while parsing macro '{0}'. No corresponding '.end_macro' found.")]
    UnendedMacro(String),

    #[error("Macro argument '{0}' was not found.")]
    ArgNotFoundMacro(String),

    /// Didn't recognize a type/directive in the `.data` directive
    /// (like `.double` or `.nothing`)
    #[error("Unrecognized data type '{0}'")]
    UnrecognizedDataType(String),

    #[error("Error while parsing float: {0}")]
    FloatError(std::num::ParseFloatError),
}

impl<'a> From<nom::Err<(&'a str, nom::error::ErrorKind)>> for ParserError {
    fn from(err: nom::Err<(&'a str, nom::error::ErrorKind)>) -> Self {
        use nom::Err as e;
        match err {
            e::Incomplete(_) => {
                unreachable!("nom::Err::Incomplete should only exist in streaming parsers")
            }
            e::Error((i, e)) => ParserError::Nom(i.into(), e),
            e::Failure((i, e)) => ParserError::Nom(i.into(), e),
        }
    }
}

#[derive(Debug, Error)]
pub enum LexerError {
    #[error("I/O Error: {0}")]
    IO(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error while parsing.\n\n{0}")]
    Parser(#[from] ParserError),
    #[error("Error while lexing the program.\n\n{0}")]
    Lexer(#[from] LexerError),
}

