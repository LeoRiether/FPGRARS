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
    /// for the file and consume it lazily. This includer will be stored in `inner`
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
        let line = strip_unneeded(&line).unwrap();

        if let Ok((_, file)) = include_directive(&line) {
            let error = format!("Can't open file: <{}>", file);
            self.inner = Some(Box::new(file_lines(file).expect(&error).parse_includes()));
            Some(String::new())
        } else {
            Some(line.into())
        }
    }
}

pub trait Includable<'a, I: Iterator<Item = String>> {
    /// Returns an iterator over RISC-V lines that can process `.include "file"` directives
    /// and flatten all of the files into one stream. Refer to
    /// [RISCVParser](../trait.RISCVParser.html#fn.parse_riscv) for example usage.
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

        self.items.next()
    }
}

pub trait MacroParseable<I: Iterator<Item = String>> {
    /// Returns an iterator that inlines macros defined in the strings.
    /// Refer to [RISCVParser](../trait.RISCVParser.html#fn.parse_riscv)
    /// for example usage.
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
