use std::{fmt, rc::Rc};

#[derive(Debug, Clone, PartialEq)]
/// Token data
pub enum Data {
    Identifier(String),
    Directive(String),
    Label(String),
    Char(char),
    Integer(i32),
    Float(f32),
    StringLiteral(String),
    CharLiteral(char),
    MacroArg(String),
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Data::Identifier(id) => write!(f, "{}", id),
            Data::Directive(d) => write!(f, "{}", d),
            Data::Label(l) => write!(f, "{}", l),
            Data::Char(c) => write!(f, "{}", c),
            Data::Integer(i) => write!(f, "{}", i),
            Data::Float(x) => write!(f, "{}", x),
            Data::StringLiteral(s) => write!(f, "{}", s),
            Data::CharLiteral(c) => write!(f, "{}", c),
            Data::MacroArg(a) => write!(f, "{}", a),
        }
    }
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
