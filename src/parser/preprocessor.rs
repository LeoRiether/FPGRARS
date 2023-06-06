use std::path::Path;

use crate::{inner_bail, parser::error::Contextualize};
use hashbrown::HashMap;
use lazy_static::lazy_static;
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

lazy_static! {
    static ref MACRO_ARGS_TIP: String = format!(
        "Maybe you forgot to call the macro with parentheses?\n   e.g. {} or {}",
        "my_macro()", "my_macro(t0, 2, \"Hello World\")"
    );
}

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
    buffer: Vec<Result<Token, Error>>,
    macros: HashMap<String, Macro>,
    equs: HashMap<String, Token>,
}

impl Preprocessor {
    pub fn new(tokens: Lexer) -> Self {
        Self {
            lexers: vec![tokens],
            buffer: Vec::new(),
            macros: HashMap::new(),
            equs: HashMap::new(),
        }
    }

    pub fn peek(&mut self) -> Option<&Result<Token, Error>> {
        let token = self.next_token()?;
        self.buffer.push(token);
        self.buffer.last()
    }

    pub fn next_token(&mut self) -> Option<Result<Token, Error>> {
        if let Some(token) = self.buffer.pop() {
            return Some(token);
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

    fn is_registered_equ(&self, name: &str) -> bool {
        self.equs.contains_key(name)
    }

    /// When a macro has been invoked in the assembly code, `expand_macro` expands the invocation,
    /// putting the body of the macro into `self.buffer`.
    fn expand_macro(&mut self, name: &str, args: &[Token]) {
        let m = self.macros.get(name).unwrap();
        let expanded_body = m.body.iter().map(|token| match token.data {
            token::Data::MacroArg(ref arg) => {
                let index = m.args[arg];
                Ok(args[index].clone())
            }
            _ => Ok(token.clone()),
        });

        for token in expanded_body {
            self.buffer.push(token);
        }
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

        let lexer = Lexer::new(path).map_err(|e| e.with_context(include_ctx))?;
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
        if let Some(Char('(')) = peek.as_ref().map(|t| &t.data) {
            let token = peek.take().unwrap();
            self.consume_macro_decl_args(&mut r#macro, token.ctx)?;
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

        r#macro.body.reverse(); // Macro bodies are stored in reverse!
        self.macros.insert(r#macro.name.clone(), r#macro);
        Ok(())
    }

    /// Reads tokens until a matching token is found.
    ///
    /// For example:
    /// ```
    /// use token::Data;
    /// let preprocessor = Lexer::from("%arg1, %arg2)".into(), "test.s");
    /// let res = preprocessor.consume_until(Data::Char(')'))
    ///
    /// let res: Vec<_> = res.unwrap().into_iter().map(|t| t.data).collect();
    /// assert_eq!(res, &[Data::MacroArg("arg1".into()), Data::MacroArg("arg2".into())]);
    /// ```
    fn consume_until(
        &mut self,
        data: token::Data,
        fallback_ctx: token::Context,
    ) -> Result<Vec<Token>, Error> {
        let mut tokens = Vec::new();
        loop {
            let token = inner_bail!(self.next_token());
            match token {
                Some(t) if t.data == data => break Ok(tokens),
                Some(t) => tokens.push(t),

                None => {
                    return Err(PreprocessorError::UnexpectedToken(None).with_context(fallback_ctx))
                }
            }
        }
    }

    /// Read macro arguments in a declaration until the closing parenthesis.
    /// Assumes the opening parenthesis has already been consumed.
    fn consume_macro_decl_args(
        &mut self,
        r#macro: &mut Macro,
        args_start_ctx: token::Context,
    ) -> Result<(), Error> {
        use super::token::Data::{Char, Identifier, MacroArg};

        let tokens = self.consume_until(Char(')'), args_start_ctx)?;

        for token in tokens {
            match token {
                Token {
                    data: MacroArg(arg),
                    ctx,
                } => {
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

                other => {
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

        Ok(())
    }

    /// Read macro arguments until the closing parenthesis.
    fn consume_macro_invocation_args(
        &mut self,
        args_start_ctx: token::Context,
    ) -> Result<Vec<Token>, Error> {
        use token::Data::Char;

        // Match opening parenthesis
        let open_paren = match self.peek() {
            None => None,
            Some(Err(_)) => None, // !
            Some(Ok(t)) => Some(t),
        };
        match open_paren.map(|t| &t.data) {
            Some(Char('(')) => {
                self.next_token();
                let tokens = self.consume_until(Char(')'), args_start_ctx)?;
                Ok(tokens)
            }
            _ => Ok(vec![]),
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
                let args = self.consume_macro_invocation_args(token.ctx.clone());
                if let Err(e) = args {
                    return Some(Err(e));
                }

                self.expand_macro(&id, &args.unwrap());
                self.next()
            }
            Identifier(id) if self.is_registered_equ(&id) => {
                let value = self.equs.get(&id).unwrap().clone();
                Some(Ok(value.with_ctx(token.ctx.clone())))
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
        // NOTE: macros are stored in reverse order!
        let testcases = &[
            (
                r#".macro INC(%rd, %rs1)
                    addi %rd, %rs1, 1
                .end_macro"#,
                "INC",
                &[
                    Integer(1),
                    MacroArg("rs1".into()),
                    MacroArg("rd".into()),
                    Identifier("addi".into()),
                ],
            ),
            (
                r#".macro NOP
                    addi x0, zero, 0
                .end_macro"#,
                "NOP",
                &[
                    Integer(0),
                    Identifier("zero".into()),
                    Identifier("x0".into()),
                    Identifier("addi".into()),
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
