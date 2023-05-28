use hashbrown::HashMap;

use crate::parser::error::Contextualize;

use super::{
    error::{Error, PreprocessorError},
    token::{self, Token},
};

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

#[derive(Debug, Default)]
struct Macro {
    name: String,
    args: HashMap<String, usize>,
    body: Vec<Token>,
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

    fn consume_include(&mut self, include_ctx: token::Context) -> Result<(), Error> {
        use super::token::Data::StringLiteral;

        let filename = match self.tokens.next() {
            Some(Token {
                data: StringLiteral(s),
                ..
            }) => s,

            other => {
                let err = if let Some(other) = other {
                    PreprocessorError::ExpectedStringLiteral(Some(other.data))
                        .with_context(other.ctx)
                } else {
                    PreprocessorError::ExpectedStringLiteral(None).with_context(include_ctx)
                };
                return Err(err);
            }
        };

        eprintln!("> including <{}>", filename);

        Ok(())
    }

    /// Read a macro until the .end_macro directive
    fn consume_macro(&mut self, macro_ctx: token::Context) -> Result<(), Error> {
        use super::token::Data::{Char, Directive, Identifier};

        let mut r#macro = Macro::default();

        // Read macro name
        r#macro.name = match self.tokens.next() {
            Some(Token {
                data: Identifier(d),
                ..
            }) => d,

            None => return Err(PreprocessorError::ExpectedMacroName(None).with_context(macro_ctx)),
            Some(other) => return Err(PreprocessorError::ExpectedMacroName(Some(other.data)).with_context(other.ctx)),
        };

        // Read macro args
        let mut peek = self.tokens.next();
        if let Some(
            token @ Token {
                data: Char('('), ..
            },
        ) = peek
        {
            self.consume_macro_args(&mut r#macro, token.ctx.clone())?;
            peek = None;
        }

        // Read macro body until .end_macro
        loop {
            match peek.take().or_else(|| self.tokens.next()) {
                Some(Token {
                    data: Directive(d), ..
                }) if d == "endmacro" || d == "end_macro" => break Ok(()),

                Some(other_token) => {
                    r#macro.body.push(other_token);
                }

                None => panic!(
                    "Macro '{0}' was not terminated by .endmacro",
                    r#macro.body[0].data
                ),
            }
        }
    }

    /// Read macro arguments until the closing parenthesis.
    fn consume_macro_args(
        &mut self,
        r#macro: &mut Macro,
        args_start_ctx: token::Context,
    ) -> Result<(), Error> {
        use super::token::Data::{Char, MacroArg};
        loop {
            match self.tokens.next() {
                Some(Token {
                    data: Char(')'), ..
                }) => break Ok(()),

                None => {
                    return Err(PreprocessorError::UnexpectedToken(None)
                        .with_context(args_start_ctx)
                        .with_tip("Did you forget to close the macro arguments with ')'?"))
                }

                Some(Token {
                    data: MacroArg(arg),
                    ctx,
                }) => {
                    use hashbrown::hash_map::Entry;
                    let index = r#macro.args.len();
                    let entry = r#macro.args.entry(arg.clone());
                    if let Entry::Occupied(_) = entry {
                        return Err(PreprocessorError::DuplicateMacroArg {
                            macro_name: r#macro.name.clone(),
                            arg,
                        }
                        .with_context(ctx));
                    }
                    
                    entry.or_insert(index);
                }

                Some(other) => {
                    return Err(PreprocessorError::UnexpectedToken(Some(other.data))
                        .with_context(other.ctx)
                        .with_tip("Here's an example of a valid macro:\n\n.macro MyMacro(%arg1, %arg2)\n  mv %arg1, %arg2\n.end_macro"))
                }
            }
        }
    }
}

impl<TI: Iterator<Item = Token>> Iterator for Preprocessor<TI> {
    type Item = Result<Token, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.tokens.next()?;
        use super::token::Data::*;

        match token.data {
            Directive(d) if d == "include" => {
                if let Err(e) = self.consume_include(token.ctx.clone()) {
                    return Some(Err(e));
                }
                self.next()
            }
            Directive(d) if d == "macro" => {
                if let Err(e) = self.consume_macro(token.ctx.clone()) {
                    return Some(Err(e));
                }
                self.next()
            }
            _ => Some(Ok(token)),
        }
    }
}
