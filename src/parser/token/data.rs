use std::fmt;

/// Token data
#[derive(Debug, Clone, PartialEq)]
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
            Data::Directive(d) => write!(f, ".{}", d),
            Data::Label(l) => write!(f, "{}:", l),
            Data::Char(c) => write!(f, "{}", c),
            Data::Integer(i) => write!(f, "{}", i),
            Data::Float(x) => write!(f, "{}", x),
            Data::StringLiteral(s) => write!(f, "\"{}\"", s),
            Data::CharLiteral(c) => write!(f, "'{}'", c),
            Data::MacroArg(a) => write!(f, "%{}", a),
        }
    }
}
