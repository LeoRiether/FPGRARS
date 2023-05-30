use owo_colors::OwoColorize;
use std::{
    fmt,
    fs::File,
    io::BufReader,
    rc::Rc,
};
use crate::utf8_lossy_lines::Utf8LossyLinesExt;

/// Token context, including the current filename, line and column
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
        for (line, i) in reader.utf8_lossy_lines().skip(from).take(3).zip(from+1..) {
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
