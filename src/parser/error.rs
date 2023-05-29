use super::{data, token};
use core::fmt;
use owo_colors::OwoColorize;
use std::{borrow::Cow, io};
use thiserror::Error;

fn some_or_eof<T: fmt::Display>(s: &Option<T>) -> Cow<'static, str> {
    match s {
        Some(s) => s.to_string().into(),
        None => Cow::Borrowed("<EOF>"),
    }
}

/// Represents any kind of error the parser may find
#[derive(Debug, Error)]
pub enum ParserError {
    /// Not the parser's fault, some std::io went wrong
    #[error("I/O Error: {0}")]
    IO(#[from] io::Error),

    #[error("Expected a register name, but found '{}'", .0.bright_blue())]
    RegisterNotFound(String),

    #[error("Value '{}' cannot be stored in data type '{1:?}'", .0.bright_blue())]
    InvalidDataType(token::Data, data::Type),

    #[error("Unknown directive '{}{}'", ".".bright_yellow(), .0.bright_yellow())]
    UnknownDirective(String),

    #[error("Unknown instruction '{}'", .0.bright_yellow())]
    UnknownInstruction(String),

    #[error("Expected a register name, but found '{}'", some_or_eof(.0).bright_yellow())]
    ExpectedRegister(Option<String>),

    #[error("Expected an immediate value, but found '{}'", some_or_eof(.0).bright_yellow())]
    ExpectedImmediate(Option<String>),

    #[error("Expected the token '{}', but found '{}' instead.", .0.bright_blue(), some_or_eof(.1).bright_yellow())]
    ExpectedToken(token::Data, Option<token::Data>),

    #[error("Did not expect token '{}' here.", some_or_eof(.0).bright_yellow())]
    UnexpectedToken(Option<token::Data>),
}

#[derive(Debug, Error)]
pub enum LexerError {
    #[error("I/O Error: {0}")]
    IO(#[from] io::Error),

    #[error("Expected '{}', but found '{}'", expected.bright_blue(), found.bright_yellow())]
    UnexpectedChar { expected: char, found: char },
}

#[derive(Debug, thiserror::Error)]
pub enum PreprocessorError {
    #[error("Expected string literal after include directive, found {}", some_or_eof(.0).bright_yellow())]
    ExpectedStringLiteral(Option<token::Data>),

    #[error("Expected macro name after .macro directive, found '{}'.", some_or_eof(.0).bright_yellow())]
    ExpectedMacroName(Option<token::Data>),

    #[error("Macro '{0}' was not terminated by .end_macro.")]
    UnterminatedMacro(String),

    #[error("The argument '{arg}' in macro '{macro_name}' was defined more than once.")]
    DuplicateMacroArg { macro_name: String, arg: String },

    #[error(
        "The argument '{arg}' in macro '{macro_name}' was used in the macro body, but not defined."
    )]
    UndefinedMacroArg { macro_name: String, arg: String },

    #[error("{} is not a valid name for an .equ. The name must be a valid identifier.", .0.bright_blue())]
    EquWithInvalidName(token::Data),
    #[error(".equ should have a name and a value: {}", ".equ <name> <value>".bright_blue())]
    UnnamedEqu,
    #[error(".equ {} has no value! Valid usage: {}", .0.bright_yellow(), ".equ <name> <value>".bright_blue())]
    EquWithNoValue(token::Data),

    #[error("Did not expect token '{}' here.", some_or_eof(.0).bright_yellow())]
    UnexpectedToken(Option<token::Data>),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Parser(#[from] ParserError),
    #[error("{0}")]
    Lexer(#[from] LexerError),
    #[error("{0}")]
    Preprocessor(#[from] PreprocessorError),
    #[error("{}\n{}", err.bold(), ctx)]
    WithContext {
        err: Box<Error>,
        ctx: token::Context,
    },
    #[error("{err}\n   {}: {tip}\n", "[tip]".bright_yellow())]
    WithTip {
        err: Box<Error>,
        tip: Cow<'static, str>,
    },
}

pub trait Contextualize {
    fn with_context(self, ctx: token::Context) -> Error;
    fn with_tip(self, tip: impl Into<Cow<'static, str>>) -> Error;
}

impl Contextualize for Error {
    fn with_context(self, ctx: token::Context) -> Error {
        match self {
            Error::WithContext { err, .. } => Error::WithContext { err, ctx },
            _ => Error::WithContext {
                err: Box::new(self),
                ctx,
            },
        }
    }

    fn with_tip(self, tip: impl Into<Cow<'static, str>>) -> Error {
        Error::WithTip {
            err: Box::new(self),
            tip: tip.into(),
        }
    }
}

macro_rules! impl_contextualize {
    ($type:ty) => {
        impl Contextualize for $type {
            fn with_context(self, ctx: token::Context) -> Error {
                Error::WithContext {
                    err: Box::new(self.into()),
                    ctx,
                }
            }

            fn with_tip(self, tip: impl Into<Cow<'static, str>>) -> Error {
                Error::WithTip {
                    err: Box::new(self.into()),
                    tip: tip.into(),
                }
            }
        }
    };
}

impl_contextualize! { LexerError }
impl_contextualize! { PreprocessorError }
impl_contextualize! { ParserError }
