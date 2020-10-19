use std::collections::VecDeque;
use std::path::PathBuf;

use super::combinators::*;
use super::util::*;

/// Generally created by calling [parse_includes](trait.Includable.html#method.parse_includes)
/// on an iterator of Strings
pub struct Includer<'a> {
    /// Stack of line iterators. Every time we encounter an .include,
    /// we push its iterator onto the stack.
    stack: Vec<Box<dyn Iterator<Item = String> + 'a>>,

    /// Stores the directory of each file include (the path but without the actual filename at the end)
    paths: Vec<PathBuf>,
}

impl<'a> Includer<'a> {
    fn pop(&mut self) {
        self.stack.pop();
        self.paths.pop();
    }
}

impl<'a> Iterator for Includer<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        // Check the last iterator of the stack until we find one that still has items
        let line = loop {
            let maybe_line = match self.stack.last_mut() {
                Some(iterator) => iterator.next(),
                None => return None,
            };

            match maybe_line {
                Some(line) => {
                    break line;
                }
                None => {
                    self.pop();
                }
            }
        };

        let line = strip_unneeded(&line).unwrap();

        if let Ok((_, file)) = include_directive(&line) {

            // Get the current path and push the filename
            let mut path = self.paths.last().unwrap().clone();
            path.push(file);

            // Push the new file line iterator onto the stack
            let error = format!("Can't open file: <{:?}>", path.to_str());
            self.stack.push(Box::new(file_lines(path.clone()).expect(&error)));

            // Push the new current path onto the stack
            path.pop();
            self.paths.push(path);

            Some(String::new())
        } else {
            Some(line.into())
        }
    }
}

pub trait Includable<'a, I: Iterator<Item = String> + 'a> {
    /// Returns an iterator over RISC-V lines that can process `.include "file"` directives
    /// and flatten all of the files into one stream. Refer to
    /// [RISCVParser](../trait.RISCVParser.html#fn.parse_riscv) for example usage.
    ///
    /// Also removes comments for some reason.
    fn parse_includes(self, filepath: PathBuf) -> Includer<'a>;
}

impl<'a, I: Iterator<Item = String> + 'a> Includable<'a, I> for I {
    fn parse_includes(self, mut filepath: PathBuf) -> Includer<'a> {
        filepath.pop(); // discard the filename
        Includer {
            stack: vec![Box::new(self)],
            paths: vec![filepath],
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
