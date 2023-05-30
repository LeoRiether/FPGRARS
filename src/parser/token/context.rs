use crate::utf8_lossy_lines::Utf8LossyLinesExt;
use owo_colors::OwoColorize;
use std::{
    fmt,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    rc::Rc,
};

/// Token context, including the current filename, line and column.
/// Displaying a context will read the file and print the 3 lines surrounding it, as well as point
/// to the position of the token.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
        } else if c == '\t' {
            self.column += 4;
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
            Path::new(&*self.file).normalize().display().bright_yellow(),
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
        for (line, i) in reader.utf8_lossy_lines().skip(from).take(3).zip(from + 1..) {
            let line = line.unwrap();
            writeln!(f, "{:^4}{} {}", i.bright_blue(), "|".bright_blue(), line)?;

            // BUG: this breaks for files over 9999 lines ¯\_(ツ)_/¯
            if i == self.line as usize {
                (0..self.column + 4).for_each(|_| write!(f, "{}", ".".bright_red()).unwrap());
                writeln!(f, "{}", "^ Here".bright_red())?;
            }
        }

        Ok(())
    }
}

pub struct ManyContexts<'a>(pub &'a Vec<Context>);
impl<'a> fmt::Display for ManyContexts<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|ctx| writeln!(f, "{}", ctx))
    }
}

//////////////////////////////////
//        Normalize Path        //
//////////////////////////////////
trait NormalizePathExt {
    fn normalize(&self) -> PathBuf;
}

// Based on [the cargo implementation](https://github.com/rust-lang/cargo/blob/fede83ccf973457de319ba6fa0e36ead454d2e20/src/cargo/util/paths.rs#L61) and the
impl NormalizePathExt for Path {
    fn normalize(&self) -> PathBuf {
        use std::path::Component::*;
        let mut components = self.components().peekable();

        let mut normalized = if let Some(c @ Prefix(..)) = components.peek().cloned() {
            components.next();
            PathBuf::from(c.as_os_str())
        } else {
            PathBuf::new()
        };

        let mut level = 0;
        for component in components {
            match component {
                Prefix(..) => unreachable!(),
                CurDir => {}
                RootDir => {
                    normalized.push(component.as_os_str());
                    level += 1;
                }
                ParentDir if level == 0 => {
                    normalized.push("..");
                }
                ParentDir => {
                    normalized.pop();
                    level -= 1;
                }
                Normal(path) => {
                    normalized.push(path);
                    level += 1;
                }
            }
        }
        normalized
    }
}
