use std::collections::VecDeque;

use hashbrown::HashMap;
use owo_colors::OwoColorize;

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
    buffer: VecDeque<Token>,
    macros: HashMap<String, Macro>,
}

impl<TI: Iterator<Item = Token>> Preprocessor<TI> {
    pub fn new(tokens: TI) -> Self {
        Self {
            tokens,
            buffer: VecDeque::new(),
            macros: HashMap::new(),
        }
    }

    fn is_registered_macro(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }

    /// When a macro has been invoked in the assembly code, `expand_macro` expands the invocation,
    /// putting the body of the macro into `self.buffer`.
    fn expand_macro(&mut self, name: &str, args: &[Token]) {
        let m = self.macros.get(name).unwrap();
        let expanded_body = m.body.iter().map(|token| match token.data {
            token::Data::MacroArg(ref arg) => {
                let index = m.args[arg];
                args[index].clone()
            }
            _ => token.clone(),
        });

        self.buffer.extend(expanded_body);
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

        // TODO: actually include files here

        Ok(())
    }

    /// Read a macro until the .end_macro directive
    fn consume_macro(&mut self, macro_ctx: token::Context) -> Result<(), Error> {
        use super::token::Data::{Char, Directive, Identifier, MacroArg};

        // Read macro name
        let name = match self.tokens.next() {
            Some(Token {
                data: Identifier(d),
                ..
            }) => d,

            None => return Err(PreprocessorError::ExpectedMacroName(None).with_context(macro_ctx)),
            Some(other) => {
                return Err(
                    PreprocessorError::ExpectedMacroName(Some(other.data)).with_context(other.ctx)
                )
            }
        };

        let mut r#macro = Macro {
            name,
            ..Macro::default()
        };

        // Read macro args
        let mut peek = self.tokens.next();
        if let Some(
            token @ Token {
                data: Char('('), ..
            },
        ) = peek
        {
            self.consume_macro_args(&mut r#macro, token.ctx)?;
            peek = None;
        }

        // Read macro body until .end_macro
        loop {
            match peek.take().or_else(|| self.tokens.next()) {
                Some(Token {
                    data: Directive(d), ..
                }) if d == "endmacro" || d == "end_macro" => break,

                Some(token) => {
                    // Make sure the argument being used was defined
                    if let MacroArg(arg) = &token.data {
                        if !r#macro.args.contains_key(arg) {
                            return Err(PreprocessorError::UndefinedMacroArg {
                                macro_name: r#macro.name.clone(),
                                arg: arg.clone(),
                            }
                            .with_context(token.ctx));
                        }
                    }

                    r#macro.body.push(token);
                }

                None => {
                    return Err(
                        PreprocessorError::UnterminatedMacro(r#macro.name).with_context(macro_ctx)
                    )
                }
            }
        }

        self.macros.insert(r#macro.name.clone(), r#macro);
        Ok(())
    }

    /// Read macro arguments until the closing parenthesis.
    fn consume_macro_args(
        &mut self,
        r#macro: &mut Macro,
        args_start_ctx: token::Context,
    ) -> Result<(), Error> {
        use super::token::Data::{Char, Identifier, MacroArg};
        loop {
            match self.tokens.next() {
                Some(Token {
                    data: Char(')'), ..
                }) => break Ok(()),

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

                None => {
                    return Err(PreprocessorError::UnexpectedToken(None)
                        .with_context(args_start_ctx)
                        .with_tip("Did you forget to close the macro arguments with ')'?"))
                }

                Some(other) => {
                    let mut err = PreprocessorError::UnexpectedToken(Some(other.data.clone()))
                        .with_context(other.ctx)
                        .with_tip(format!("{}:\n   .macro MyMacro(%arg1, %arg2)\n       mv %arg1, %arg2\n   .end_macro", "Here's an example of a valid macro".bold()));

                    if let Identifier(id) = &other.data {
                        err = err.with_tip(format!(
                            "Maybe you forgot to put a {0} before the argument name? e.g. '{0}{1}'",
                            "%".bright_yellow(),
                            id.bright_yellow()
                        ));
                    }

                    return Err(err);
                }
            }
        }
    }
}

impl<TI: Iterator<Item = Token>> Iterator for Preprocessor<TI> {
    type Item = Result<Token, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(token) = self.buffer.pop_front() {
            return Some(Ok(token));
        }

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
            Identifier(id) if self.is_registered_macro(&id) => {
                // TODO: get_macro_args
                // let args = self.get_macro_args();
                self.expand_macro(&id, &[]);
                self.next()
            }
            _ => Some(Ok(token)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::lexer::Lexer;

    #[test]
    fn test_macros() {
        use crate::parser::token::Data::*;
        let testcases = &[
            (
                r#".macro INC(%rd, %rs1)
                    addi %rd, %rs1, 1
                .end_macro"#,
                "INC",
                &[
                    Identifier("addi".into()),
                    MacroArg("rd".into()),
                    MacroArg("rs1".into()),
                    Integer(1),
                ],
            ),
            (
                r#".macro NOP
                    addi x0, zero, 0
                .end_macro"#,
                "NOP",
                &[
                    Identifier("addi".into()),
                    Identifier("x0".into()),
                    Identifier("zero".into()),
                    Integer(0),
                ],
            ),
        ];

        for (input, macro_name, expected_macro) in testcases {
            let tokens = Lexer::from_content(String::from(*input), "macro.s");
            let mut preprocessor = Preprocessor::new(tokens);
            for token in preprocessor.by_ref() {
                assert!(token.is_ok());
            }

            assert!(preprocessor.is_registered_macro(macro_name));
            let m = preprocessor.macros.get(*macro_name).unwrap();
            let m: Vec<_> = m.body.iter().map(|t| t.data.clone()).collect();

            assert_eq!(m, *expected_macro);
        }
    }
}
