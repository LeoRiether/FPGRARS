use fnv::FnvHashMap;
use std::mem;
use std::path::PathBuf;

use super::combinators::*;
use super::util::*;

/// Generally created by calling [parse_includes](trait.Includable.html#method.parse_includes)
/// on an iterator of Strings
// TODO: check for ciclic includes, preferably in a better way than RARS
// (we should allow a file to be included more than once, maybe?)
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
            self.stack
                .push(Box::new(file_lines(path.clone()).expect(&error)));

            // Push the new current path onto the stack
            path.pop();
            self.paths.push(path);

            self.next()
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

/// We store each line of a parsed macro in a similar manner to JavaScript's template strings.
/// When the arguments are applied (in [build()](struct.MacroLine.html#method.build)),
/// we output the concatenation `{ raw[0], arg[0], raw[1], arg[1], ..., raw[n-1], arg[n-1], raw[n] }`
#[derive(Debug, Default, PartialEq, Eq)]
struct MacroLine {
    raw: Vec<String>,
    args: Vec<usize>,
}

impl MacroLine {
    fn from_string(s: &str, arg_names: &FnvHashMap<String, usize>) -> Result<Self, Error> {
        use nom::bytes::complete::take_till;
        let take_raw = |s| take_till::<_, _, ()>(|c| c == '%')(s).unwrap();
        let take_arg = |s| take_till::<_, _, ()>(|c| is_separator(c) || c == '(' || c == ')')(s).unwrap();

        let mut res = Self::default();

        fn ignore_char(t: &str) -> &str {
            if t.len() >= 1 {
                &t[1..]
            } else {
                t
            }
        }

        let (mut s, prefix) = take_raw(s);
        s = ignore_char(s); // ignore the %
        res.raw.push(prefix.into());

        while s.len() > 0 {
            let (rest, arg) = take_arg(s);
            let (rest, raw) = take_raw(rest);
            s = ignore_char(rest);

            let arg_index = match arg_names.get(arg) {
                Some(x) => *x,
                None => return Err(Error::ArgNotFoundMacro(arg.to_owned())),
            };

            res.args.push(arg_index);
            res.raw.push(raw.into());
        }

        Ok(res)
    }

    /// Builds a single line, replacing where arguments were by the actual values
    fn build(&self, args: &[String]) -> String {
        let mut ans = String::new();

        for (r, &p) in self.raw.iter().zip(self.args.iter()) {
            ans.extend(r.chars());
            ans.extend(args[p].chars());
        }

        ans.extend(self.raw.last().unwrap().chars());
        ans
    }
}

/// Exists while we're inside a `.macro`, `.end_macro` definition, just to keep
/// track of the argument names. After that we can discard arg_names and keep only thea
/// lines: this is a [Macro](struct.Macro.html)
struct MacroBuilder {
    /// Maps a argument string to its index in the macro declaration
    arg_names: FnvHashMap<String, usize>,

    /// Stack of macro lines
    lines: Vec<MacroLine>,

    name: String,
}

impl MacroBuilder {
    fn new(name: String, arg_names: Vec<String>) -> Self {
        Self {
            arg_names: arg_names
                .into_iter()
                .enumerate()
                .map(|(i, s)| (s, i))
                .collect(),
            lines: Vec::new(),
            name,
        }
    }

    fn push_line(&mut self, s: &str) -> Result<(), Error> {
        self.lines.push(MacroLine::from_string(s, &self.arg_names)?);
        Ok(())
    }

    fn to_macro(self) -> Macro {
        // We reverse the lines so we can get them in stack order later
        Macro {
            lines: self.lines.into_iter().rev().collect(),
        }
    }
}

/// Represents a parsed macro.
struct Macro {
    /// Stack of macro lines
    lines: Vec<MacroLine>,
}

impl Macro {
    /// Builds a stack of macro lines by building every line with [MacroLine.build](struct.MacroLine.html#method.build)
    fn build(&self, args: &[String]) -> Vec<String> {
        self.lines.iter().map(|m| m.build(args)).collect()
    }
}

/// Generally created calling [parse_macros](trait.MacroParseable.html#method.parse_macros)
/// on an iterator of Strings
pub struct MacroParser<I>
where
    I: Iterator<Item = String>,
{
    items: I,

    /// Stack of lines we should process before consuming items
    buf: Vec<String>,

    macros: FnvHashMap<(String, usize), Macro>,
    eqvs: FnvHashMap<String, String>,
}

impl<I: Iterator<Item = String>> MacroParser<I> {
    /// Parses a `.macro NAME(%args)` declaration and, if it encounters it, returns a MacroBuilder
    fn parse_macro_declaration(&self, s: &str) -> Option<MacroBuilder> {
        declare_macro(s)
            .ok()
            .map(|(_, (name, args))| MacroBuilder::new(name, args))
    }

    /// Consumes the lines until we find an `.end_macro`
    fn parse_until_end(
        &mut self,
        mut builder: MacroBuilder,
    ) -> Result<((String, usize), Macro), Error> {
        loop {
            match self.items.next() {
                Some(line) if end_macro(&line) => {
                    let arg_count = builder.arg_names.len();
                    let name = mem::replace(&mut builder.name, String::new());
                    return Ok(((name, arg_count), builder.to_macro()));
                }
                None => return Err(Error::UnendedMacro(builder.name)),

                Some(line) => builder.push_line(&line)?,
            };
        }
    }

    /// Parses a macro usage and optionally returns the lines to be inlined
    fn parse_macro_use(&self, s: &str) -> Option<Vec<String>> {
        let (s, label) = nom::combinator::opt(parse_label)(s).unwrap();
        let label = label.map(|l| format!("{}:", l));

        let (_, (name, args)) = macro_use(s).ok()?;
        let key = (name, args.len());
        self.macros.get(&key).map(|m| m.build(&args)).map(|mut v| {
            v.extend(label);
            v
        })
    }

    // TODO: this function copies every line, even when it doesn't find
    // any eqvs, which is most of the time. We should optimize it a bit
    // it also replaces matches inside a string, which is not desirable
    /// Bad functon in dire need of a rewrite. Replaces eqvs by their correspondents
    /// in an inneficient manner and replaces stuff it shouldn't. Will do for now.
    fn replace_eqvs(&self, s: String) -> String {
        // There can't be any eqvs
        if self.eqvs.len() == 0 {
            return s;
        }

        let is_token = |c| !is_separator(c) && c != '(' && c != ')';
        let mut buf = String::new();
        let mut ans = String::new();
        let mut found_eqv = false;

        let mut push_buf = |buf: &mut String, ans: &mut String| {
            if buf.len() > 0 {
                let eqv_to = self.eqvs.get(buf);
                found_eqv = found_eqv || eqv_to.is_some();
                ans.push_str(eqv_to.unwrap_or(&buf));
                buf.clear();
            }
        };

        for c in s.chars() {
            match is_token(c) {
                true => buf.push(c),
                false => {
                    push_buf(&mut buf, &mut ans);
                    ans.push(c);
                }
            }
        }
        push_buf(&mut buf, &mut ans);

        // we won't support eqv aliasing for now, I might implement them whenever I feel like
        // detecting some eqv cycles
        ans
    }
}

impl<I: Iterator<Item = String>> Iterator for MacroParser<I> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let line = match self.buf.pop() {
            Some(line) => line,
            None => self.items.next()?,
        };

        // Is the line a macro declaration?
        if let Some(builder) = self.parse_macro_declaration(&line) {
            let (key, parsed_macro) = self.parse_until_end(builder).unwrap();
            self.macros.insert(key, parsed_macro);
            return self.next();
        }

        // Is the line a macro usage?
        if let Some(inlined) = self.parse_macro_use(&line) {
            self.buf.extend(inlined);
            return self.next();
        }

        // Is the line an eqv declaration?
        if let Ok((_, (key, value))) = declare_eqv(&line) {
            self.eqvs.insert(key, value);
            return self.next();
        }

        Some(self.replace_eqvs(line))
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
            buf: Vec::new(),
            macros: FnvHashMap::default(),
            eqvs: FnvHashMap::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macros() {
        let mut builder = MacroBuilder::new("Bob".into(), vec!["arg1".into(), "arg2".into()]);
        builder.push_line("li %arg1 10").unwrap();
        builder.push_line("%arg2").unwrap();

        let m = builder.to_macro();

        // notice the lines are in stack order
        assert_eq!(
            m.lines,
            vec![
                MacroLine {
                    raw: vec!["".into(), "".into()],
                    args: vec![1]
                },
                MacroLine {
                    raw: vec!["li ".into(), " 10".into()],
                    args: vec![0]
                },
            ]
        );
    }
}
