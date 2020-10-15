use nom::{character::complete::space0, sequence::preceded, AsBytes};
use std::collections::VecDeque;

use super::combinators::*;
use super::util::*;

/// Generally created by calling [parse_includes](trait.Includable.html#method.parse_includes)
/// on an iterator of Strings
pub struct Includer<'a, I>
where
    I: Iterator<Item = String>,
{
    lines: I,

    /// If we encounter an `.include "file"` in a line, we will create a includer
    /// for the file and consume it lazily. This includerwill be stored in `inner`
    inner: Option<Box<dyn Iterator<Item = String> + 'a>>,
}

impl<'a, I: Iterator<Item = String>> Iterator for Includer<'a, I> {
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

        let line = self.lines.next()?;
        if let Ok((_, file)) = include_directive(line.as_bytes()) {
            let error = format!("Can't open file: {}", file);
            self.inner =
                Some(Box::new(file_lines(file).expect(&error)));
            Some(String::new())
        } else {
            Some(line)
        }
    }
}

pub trait Includable<'a, I: Iterator<Item = String>> {
    /// Returns an iterator over RISC-V lines that can process `.include "file"` directives
    /// and flatten all of the files into one stream.
    ///
    /// Also removes comments for some reason.
    fn parse_includes(self) -> Includer<'a, I>;
}

impl<'a, I: Iterator<Item = String>> Includable<'a, I> for I {
    fn parse_includes(self) -> Includer<'a, I> {
        Includer {
            lines: self,
            inner: None,
        }
    }
}

/// Generally created calling [parse_macros](trait.MacroParseable.html#method.parse_macros)
/// on an iterator of Strings
pub struct MacroParser<I>
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
