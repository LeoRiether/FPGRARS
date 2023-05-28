use core::fmt;
use std::{borrow::Cow, io};
use thiserror::Error;

use super::{
    data,
    token,
};

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

    #[error("Expected a register name, but found '{0}'")]
    RegisterNotFound(String),

    /// Didn't recognize a type/directive in the `.data` directive
    /// (like `.double` or `.nothing`)
    #[error("Unrecognized data type '{0}'")]
    UnrecognizedDataType(String),

    #[error("Value '{0}' cannot be stored in data type '{1:?}'")]
    InvalidDataType(token::Data, data::Type),
}

#[derive(Debug, Error)]
pub enum LexerError {
    #[error("I/O Error: {0}")]
    IO(#[from] io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum PreprocessorError {
    #[error("Expected string literal after include directive, found {}", some_or_eof(.0))]
    ExpectedStringLiteral(Option<token::Data>),

    #[error("Expected macro name after .macro directive, found '{}'.\n\nExample: .macro Name(%arg1, %arg2)\n  add %arg1, %arg1, %arg2\n.end_macro", some_or_eof(.0))]
    ExpectedMacroName(Option<token::Data>),

    #[error("Macro '{0}' was not terminated by .end_macro.\n\nExample: .macro Name(%arg1, %arg2)\n  add %arg1, %arg1, %arg2\n.end_macro")]
    UnterminatedMacro(String),

    #[error("The argument '{arg}' in macro '{macro_name}' was defined more than once.")]
    DuplicateMacroArg { macro_name: String, arg: String },

    #[error("Did not expect token '{}' here.", some_or_eof(.0))]
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
    #[error("{err}")]
    WithContext {
        err: Box<Error>,
        ctx: token::Context,
    },
    #[error("{err}")]
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
