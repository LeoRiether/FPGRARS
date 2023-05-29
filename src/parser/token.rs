use owo_colors::OwoColorize;
use std::{fmt, fs::File, io::{self, BufReader, BufRead}, rc::Rc};

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

impl Data {
    pub fn extract_u32(&self) -> Option<u32> {
        match self {
            Data::Integer(i) => Some(*i as u32),
            Data::CharLiteral(c) => Some(*c as u32),
            _ => None,
        }
    }

    pub fn extract_f32(&self) -> Option<u32> {
        match self {
            Data::Float(f) => Some(*f as u32),
            _ => None,
        }
    }
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

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "   {} {} at line {}, column {}",
            "-->".bright_blue().bold(),
            self.file.bright_yellow(),
            self.line.bright_yellow(),
            self.column.bright_yellow(),
        )?;

        let file = File::open(&*self.file);
        if let Err(e) = file {
            return writeln!(
                f, 
                "   While we were printing this error another error ocurred!\n   Couldn't open '{}' because: {}",
                self.file.bright_yellow(), e.bold()
            );
        }

        let reader = BufReader::new(file.unwrap());
        let from = self.line.saturating_sub(2) as usize;
        for (line, i) in reader.lines().skip(from).take(3).zip(from+1..) {
            let line = line.unwrap();
            writeln!(f, "{:3} {} {}", i.bright_blue(), "|".bright_blue(), line)?;

            // BUG: this breaks for files over 999 lines ¯\_(ツ)_/¯
            if i == self.line as usize {
                (0..self.column + 6).for_each(|_| write!(f, "{}", ".".bright_red()).unwrap());
                writeln!(f, "{}", "^ Here".bright_red())?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
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
