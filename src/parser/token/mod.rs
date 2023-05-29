pub mod context;
pub mod data;
pub use context::{Context, ManyContexts};
pub use data::Data;

use super::error::{Contextualize, Error};

/// Token given by the lexer
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub data: Data,
    pub ctx: Context,
}

impl Token {
    pub fn new(data: Data) -> Self {
        Self {
            data,
            ctx: Context::empty(),
        }
    }

    pub fn with_ctx(mut self, ctx: Context) -> Self {
        self.ctx = ctx;
        self
    }
}

pub trait ContextualizeResult {
    fn with_ctx(self, ctx: Context) -> Self;
}

impl ContextualizeResult for Result<Token, Error> {
    fn with_ctx(self, ctx: Context) -> Self {
        match self {
            Ok(token) => Ok(token.with_ctx(ctx)),
            Err(e) => Err(e.with_context(ctx)),
        }
    }
}
