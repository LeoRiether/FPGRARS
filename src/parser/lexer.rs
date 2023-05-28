//! TODO: replace unwraps and panics in this file by proper error handling

use std::fs;

use super::token::{Context, Data, Token};

// TODO: what about $, @, etc?
macro_rules! allowed_identifier {
    () => {
        'a'..='z' | 'A'..='Z' | '_' | '0'..='9'
    };
    (start) => {
        'a'..='z' | 'A'..='Z' | '_'
    };
}

#[derive(Debug)]
/// Iterator over the tokens of a RISC-V file. Also see [`Token`]
pub struct Lexer {
    /// Content of the file the Lexer is lexing
    content: String,
    /// Current position in the content string
    cursor: usize,
    /// Context of the current token
    context: Context,
}

impl Lexer {
    // TODO: should return Result<Self> if reading from the file fails
    pub fn new(entry_file: &str) -> Self {
        let buf = fs::read(entry_file).unwrap();
        let content = String::from_utf8_lossy(&buf).to_string();
        Self::from_content(content, entry_file)
    }

    pub fn from_content(content: String, filename: &str) -> Self {
        Self {
            content,
            cursor: 0,
            context: Context::new(filename),
        }
    }

    pub fn peek(&self) -> Option<char> {
        self.content.as_str().get(self.cursor..)?.chars().next()
    }

    pub fn consume(&mut self) -> Option<char> {
        let mut chars = self.content.as_str().get(self.cursor..)?.char_indices();
        let (_, next_char) = chars.next()?;

        // TODO: when it's stabilized, this can be replaced by `self.cursor = chars.offset`
        match chars.next().map(|(i, _)| i) {
            Some(offset) => self.cursor += offset,
            None => self.cursor = self.content.len(),
        }

        self.context.advance_char(next_char);
        Some(next_char)
    }

    pub fn consume_comment(&mut self) {
        while !matches!(self.consume(), None | Some('\n')) {
            // continue consuming
        }
    }

    fn next_identifier(&mut self) -> Token {
        let mut id = String::new();
        while let Some(allowed_identifier!()) = self.peek() {
            let c = self.consume().unwrap();
            id.push(c);
        }

        Token::new(Data::Identifier(id))
    }

    // WARN: assumes the '\' has already been consumed
    fn next_escape_sequence(&mut self) -> char {
        match self.consume() {
            Some('n') => '\n',
            Some('t') => '\t',
            Some('r') => '\r',
            Some('\\') => '\\',
            Some('"') => '"',
            _ => todo!("Return an Error::InvalidEscapeSequence"),
        }
    }

    fn next_string_literal(&mut self) -> Token {
        let mut string = String::new();
        self.consume().unwrap(); // TODO: assert it's a "
        while self.peek() != Some('"') {
            let mut c = self.consume().unwrap();
            if c == '\\' {
                c = self.next_escape_sequence();
            }
            string.push(c);
        }
        self.consume().unwrap(); // TODO: assert it's a "

        Token::new(Data::StringLiteral(string))
    }

    fn next_char_literal(&mut self) -> Token {
        let mut c = self.consume().unwrap();
        self.consume().unwrap(); // TODO: assert it's a '
        if c == '\\' {
            c = self.next_escape_sequence();
        }
        self.consume().unwrap(); // TODO: assert it's a '

        Token::new(Data::CharLiteral(c))
    }

    // TODO: float support
    fn next_number(&mut self) -> Token {
        let cursor = self.cursor;
        let mut i = 0;
        while let Some('0'..='9' | 'x' | 'b' | 'd') = self.peek() {
            self.consume().unwrap();
            i += 1; // Those characters are guaranteed to be ASCII!
        }

        let mut slice = &self.content[cursor..cursor + i];

        let mut negative = false;
        if let Some(positive_part) = slice.strip_prefix('-') {
            slice = positive_part;
            negative = true;
        }

        let res = if let Some(slice) = slice.strip_prefix("0x") {
            i32::from_str_radix(slice, 16)
        } else if let Some(slice) = slice.strip_prefix("0b") {
            i32::from_str_radix(slice, 2)
        } else if let Some(slice) = slice.strip_prefix("0d") {
            slice.parse::<i32>()
        } else {
            slice.parse::<i32>()
        };

        let mut x = res.unwrap();
        if negative {
            x = -x;
        }
        Token::new(Data::Integer(x))
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let ctx = self.context.clone();

        let next_char = match self.peek() {
            None => return None,
            Some(c) => c,
        };

        match next_char {
            // whitespace
            ' ' | ',' | '\n' | '\t' | '\x09'..='\x0d' => {
                self.consume().unwrap();
                self.next()
            }

            // comments
            '#' => {
                self.consume().unwrap();
                self.consume_comment();
                self.next()
            }

            '.' => {
                self.consume().unwrap();
                let id = match self.next_identifier().data {
                    Data::Identifier(id) => id,
                    _ => unreachable!(),
                };
                Some(Token::new(Data::Directive(id)).with_ctx(ctx))
            }

            '%' => {
                self.consume().unwrap();
                let id = match self.next_identifier().data {
                    Data::Identifier(id) => id,
                    _ => unreachable!(),
                };
                Some(Token::new(Data::MacroArg(id)).with_ctx(ctx))
            }

            '"' => Some(self.next_string_literal().with_ctx(ctx)),
            '\'' => Some(self.next_char_literal().with_ctx(ctx)),
            ':' | '(' | ')' => {
                self.consume().unwrap();
                Some(Token::new(Data::Char(next_char)).with_ctx(ctx))
            }

            '-' | '0'..='9' => Some(self.next_number().with_ctx(ctx)),

            allowed_identifier!(start) => Some(self.next_identifier().with_ctx(ctx)),

            _ => panic!("Unimplemented character: {}", next_char),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_creation() {
        let lexer = Lexer::from_content(String::default(), "test1.s");
        let tokens = lexer.map(|t| t.data).collect::<Vec<_>>();
        assert_eq!(tokens, &[]);
    }

    #[test]
    fn test_chars() {
        let mut lexer = Lexer::from_content(String::from("abc"), "test.s");
        assert_eq!(lexer.peek(), Some('a'));
        assert_eq!(lexer.consume(), Some('a'));
        assert_eq!(lexer.peek(), Some('b'));
        assert_eq!(lexer.consume(), Some('b'));
        assert_eq!(lexer.consume(), Some('c'));
        assert_eq!(lexer.consume(), None);
    }

    #[test]
    fn test_lexer_basic() {
        let data = r#"
.include "another_file.s"
.text 
main:
    li a0 1
    add t0, t1, t2
    add 1, 2, 0(a4)
"#;
        let lexer = Lexer::from_content(String::from(data), "test1.s");
        let tokens = lexer.map(|t| t.data).collect::<Vec<_>>();

        use crate::parser::token::Data::*;
        assert_eq!(
            tokens,
            &[
                Directive("include".into()),
                StringLiteral("another_file.s".into()),
                Directive("text".into()),
                Identifier("main".into()),
                Char(':'),
                Identifier("li".into()),
                Identifier("a0".into()),
                Integer(1),
                Identifier("add".into()),
                Identifier("t0".into()),
                Identifier("t1".into()),
                Identifier("t2".into()),
                Identifier("add".into()),
                Integer(1),
                Integer(2),
                Integer(0),
                Char('('),
                Identifier("a4".into()),
                Char(')'),
            ]
        );
    }

    #[test]
    fn test_lexer_macro() {
        let data = r#"
.macro DE1(%reg,%salto)
	li %reg, 0x10008000	# carrega tp
	bne gp, %reg, %salto	# Na DE1 gp = 0 ! Não tem segmento .extern
.end_macro

DE1(t0, LABEL)
"#;
        let lexer = Lexer::from_content(String::from(data), "test_macros.s");
        let tokens = lexer.map(|t| t.data).collect::<Vec<_>>();

        use crate::parser::token::Data::*;
        assert_eq!(
            tokens,
            &[
                Directive("macro".into()),
                Identifier("DE1".into()),
                Char('('),
                MacroArg("reg".into()),
                MacroArg("salto".into()),
                Char(')'),
                Identifier("li".into()),
                MacroArg("reg".into()),
                Integer(0x10008000),
                Identifier("bne".into()),
                Identifier("gp".into()),
                MacroArg("reg".into()),
                MacroArg("salto".into()),
                Directive("end_macro".into()),
                Identifier("DE1".into()),
                Char('('),
                Identifier("t0".into()),
                Identifier("LABEL".into()),
                Char(')'),
            ]
        );
    }

    #[test]
    fn test_corner_cases() {
        let lexer = Lexer::from_content(String::from("LABEL_ONE: nop"), "test_corner.s");
        let tokens = lexer.map(|t| t.data).collect::<Vec<_>>();
        use crate::parser::token::Data::*;
        assert_eq!(
            tokens,
            &[
                Identifier("LABEL_ONE".into()),
                Char(':'),
                Identifier("nop".into())
            ]
        );
    }
}