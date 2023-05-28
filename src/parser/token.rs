use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
/// Token data
pub enum Data {
    Identifier(String),
    Directive(String),
    Char(char),
    Integer(i32),
    Float(f32),
    StringLiteral(String),
    CharLiteral(char),
    MacroArg(String),
}

#[derive(Debug, Clone, PartialEq)]
/// Token context, including the current filename, line and column
pub struct Context {
    pub file: Rc<String>,
    pub line: u32,
    pub column: u32,
}

impl Context {
    pub fn new(file: &str) -> Self {
        Self {
            file: Rc::new(file.to_owned()),
            line: 1,
            column: 1,
        }
    }

    pub fn empty() -> Self {
        Self {
            file: Rc::new(String::new()),
            line: 0,
            column: 0,
        }
    }

    pub fn advance_char(&mut self, c: char) {
        if c == '\n' {
            self.column = 1;
            self.line += 1;
        } else {
            self.column += 1;
        }
    }
}

#[derive(Debug, PartialEq)]
/// Token given by the lexer
pub struct Token {
    pub data: Data,
    pub ctx: Context,
}

impl Token {
    pub fn new(data: Data) -> Self {
        Self { data, ctx: Context::empty() }
    }

    pub fn with_ctx(mut self, ctx: Context) -> Self {
        self.ctx = ctx;
        self
    }
}
