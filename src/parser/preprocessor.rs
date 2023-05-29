use std::{collections::VecDeque, path::Path};

use crate::{inner_bail, parser::error::Contextualize};
use hashbrown::HashMap;
use owo_colors::OwoColorize;

use super::{
    error::{Error, PreprocessorError},
    lexer::Lexer,
    token::{self, Token},
};

static MACRO_EXAMPLE_TIP: &str =
    "\x1b[1mHere's an example of a macro using arguments correctly:\x1b[0m
   .macro Name(%arg1, %arg2)
       add %arg1, %arg1, %arg2
   .end_macro";

/// Defines the `preprocess` methods for a lexer
/// ```
/// let tokens = Lexer::new("riscv.s").preprocess();
/// ```
pub trait Preprocess {
    fn preprocess(self) -> Preprocessor;
}

impl Preprocess for Lexer {
    fn preprocess(self) -> Preprocessor {
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
pub struct Preprocessor {
    /// Stack of lexers. When we find an `.include` directive, we push a new lexer onto the stack.
    lexers: Vec<Lexer>,
    buffer: VecDeque<Token>,
    macros: HashMap<String, Macro>,
    equs: HashMap<String, Token>,
}

impl Preprocessor {
    pub fn new(tokens: Lexer) -> Self {
        Self {
            lexers: vec![tokens],
            buffer: VecDeque::new(),
            macros: HashMap::new(),
            equs: HashMap::new(),
        }
    }

    pub fn next_token(&mut self) -> Option<Result<Token, Error>> {
        if let Some(token) = self.buffer.pop_front() {
            return Some(Ok(token));
        }

        loop {
            let lexer = self.lexers.last_mut()?;
            if let Some(token) = lexer.next() {
                return Some(token);
            }
            self.lexers.pop();
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

        let include_path = match inner_bail!(self.next_token()).map(|t| t.data) {
            Some(StringLiteral(s)) => s,

            other => {
                let err = PreprocessorError::ExpectedStringLiteral(other)
                    .with_context(include_ctx)
                    .with_tip(format!(
                        "The correct usage is {}",
                        ".include \"filename.s\"".bright_blue()
                    ));
                return Err(err);
            }
        };

        let path = Path::new(self.lexers.last().unwrap().file().as_str())
            .parent()
            .unwrap()
            .join(include_path);
        let path = path
            .as_os_str()
            .to_str()
            .unwrap_or_else(|| panic!("Path is not valid UTF-8: {}", path.display().bright_red()));

        let lexer = Lexer::new(path);
        self.lexers.push(lexer);
        Ok(())
    }

    /// Read a macro until the .end_macro directive
    fn consume_macro(&mut self, macro_ctx: token::Context) -> Result<(), Error> {
        use super::token::Data::{Char, Directive, Identifier, MacroArg};

        // Read macro name
        let token = inner_bail!(self.next_token());
        let name = match token.as_ref().map(|t| &t.data) {
            Some(Identifier(d)) => d,

            _ => {
                let ctx = token.as_ref().map(|t| t.ctx.clone()).unwrap_or(macro_ctx);
                return Err(PreprocessorError::ExpectedMacroName(token.map(|t| t.data))
                    .with_context(ctx)
                    .with_tip(MACRO_EXAMPLE_TIP));
            }
        };

        let mut r#macro = Macro {
            name: name.to_string(),
            ..Macro::default()
        };

        // Read macro args
        let mut peek = inner_bail!(self.next_token());
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
            let token = match peek.take() {
                Some(token) => Some(token),
                None => inner_bail!(self.next_token()),
            };

            match token {
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
                            .with_context(token.ctx)
                            .with_tip(MACRO_EXAMPLE_TIP));
                        }
                    }

                    r#macro.body.push(token);
                }

                None => {
                    return Err(PreprocessorError::UnterminatedMacro(r#macro.name)
                        .with_context(macro_ctx)
                        .with_tip(MACRO_EXAMPLE_TIP))
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
            match inner_bail!(self.next_token()) {
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
                        .with_context(ctx)
                        .with_tip(MACRO_EXAMPLE_TIP));
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
                        .with_tip(MACRO_EXAMPLE_TIP);

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

    /// Read an .equ
    fn consume_equ(&mut self, ctx: token::Context) -> Result<(), Error> {
        use token::Data::Identifier;

        let (name, value) = (self.next_token(), self.next_token());
        let name = inner_bail!(name).map(|t| t.data);
        let value = inner_bail!(value);

        match (name, value) {
            (Some(Identifier(name)), Some(value)) => {
                self.equs.insert(name, value);
                Ok(())
            }

            (Some(name), None) => Err(PreprocessorError::EquWithNoValue(name).with_context(ctx)),
            (None, _) => Err(PreprocessorError::UnnamedEqu.with_context(ctx)),
            (Some(other_token), _) => {
                Err(PreprocessorError::EquWithInvalidName(other_token).with_context(ctx))
            }
        }
    }
}

impl Iterator for Preprocessor {
    type Item = Result<Token, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let token = match self.next_token()? {
            Ok(t) => t,
            Err(e) => return Some(Err(e)),
        };

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
            Directive(d) if d == "equ" || d == "eqv" => {
                if let Err(e) = self.consume_equ(token.ctx.clone()) {
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
