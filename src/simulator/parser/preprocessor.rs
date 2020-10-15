use std::collections::VecDeque;

use super::combinators::*;
use super::util::*;
use combine::{
    self,
    parser::{
        char::{spaces, string},
        sequence::between,
        token,
    },
    Parser,
};

/// Generally created by calling [parse_includes](trait.Includable.html#method.parse_includes)
/// on an iterator of Strings
struct Includer<I>
where
    I: Iterator<Item = String>,
{
    lines: I,

    /// If we encounter an `.include "file"` in a line, we will create a includer
    /// for the file and consume it lazily. This includerwill be stored in `inner`
    inner: Option<Box<dyn Iterator<Item = String>>>,
}

impl<I: Iterator<Item = String>> Iterator for Includer<I> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        // If there's an inner iterator, consume that first
        if let Some(inner) = &mut self.inner {
            if let Some(line) = inner.next() {
                return Some(line);
            } else {
                self.inner = None;
            }
        }

        // Parse an `.include`
        let line = self.lines.next()?;
        let line = line.as_str();
        let line = spaces()
            .with(remove_comment())
            .parse(line)
            .map(|(x, _): (String, _)| x).unwrap();

        let mut include_parser = spaces()
            .with(string(".include"))
            .with(spaces())
            .with(quoted_string());

        let line = line.as_str();
        if let Ok((file, _)) = include_parser.parse(line) {
            self.inner = Some(Box::new(file_lines(file.as_ref()).unwrap()));
            Some("".to_string()) // not sure this is ideal
        } else {
            Some(line.into())
        }
    }
}

pub trait Includable<I: Iterator<Item = String>> {
    /// Returns an iterator over RISC-V lines that can process `.include "file"` directives
    /// and flatten all of the files into one stream.
    ///
    /// Also removes comments for some reason.
    fn parse_includes(self) -> Includer<I>;
}

impl<I: Iterator<Item = String>> Includable<I> for I {
    fn parse_includes(self) -> Includer<I> {
        Includer {
            lines: self,
            inner: None,
        }
    }
}

/// Generally created calling [parse_macros](trait.MacroParseable.html#method.parse_macros)
/// on an iterator of Strings
struct MacroParser<I>
where
    I: Iterator<Item = String>,
{
    items: I,
    buf: VecDeque<String>,
}

impl<I: Iterator<Item = String>> Iterator for MacroParser<I> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        // Is there anything in the buffer?
        if let Some(line) = self.buf.pop_front() {
            return Some(line);
        }

        let line = self.items.next()?;

        Some(line.into())
    }
}

pub trait MacroParseable<I: Iterator<Item = String>> {
    /// Returns an iterator that defines macros when they're defined
    /// and inlines them when they appear in the RISC-V code
    fn parse_macros(self) -> MacroParser<I>;
}

impl<I: Sized + Iterator<Item = String>> MacroParseable<I> for I {
    fn parse_macros(self) -> MacroParser<I> {
        MacroParser {
            items: self,
            buf: VecDeque::new(),
        }
    }
}
