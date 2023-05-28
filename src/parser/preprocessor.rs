use super::token::Token;

/// Defines the `preprocess` methods for iterators of tokens.
/// ```
/// let tokens = Lexer::new("riscv.s").preprocess();
/// ```
pub trait Preprocess<TI: Iterator<Item = Token>> {
    fn preprocess(self) -> Preprocessor<TI>;
}

impl<TI: Iterator<Item = Token>> Preprocess<TI> for TI {
    fn preprocess(self) -> Preprocessor<TI> {
        Preprocessor::new(self)
    }
}

/// A preprocessor for RISC-V assembly files that supports includes, macros and equs.
/// Generally constructed by calling the [`Preprocess::preprocess`] method.
pub struct Preprocessor<TI: Iterator<Item = Token>> {
    tokens: TI,
}

impl<TI: Iterator<Item = Token>> Preprocessor<TI> {
    pub fn new(tokens: TI) -> Self {
        Self { tokens }
    }
}

impl<TI: Iterator<Item = Token>> Iterator for Preprocessor<TI> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokens.next()
    }
}
