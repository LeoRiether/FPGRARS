use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

/// Represents any kind of error the parser may find
#[derive(Debug)]
pub enum Error {
    /// Not the parser's fault, some std::io went wrong
    IO(io::Error),

    LabelNotFound(String),
    Nom(String, nom::error::ErrorKind), // I'm feeling lazy
    RegisterNotFound(String),
    InstructionNotFound(String),

    UnendedMacro(String),
    ArgNotFoundMacro(String),

    /// Didn't recognize a type/directive in the `.data` directive
    /// (like `.double` or `.nothing`)
    UnrecognizedDataType(String),
    FloatError(std::num::ParseFloatError),

    OnLine(String, Box<Error>),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IO(e)
    }
}

impl<'a> From<nom::Err<(&'a str, nom::error::ErrorKind)>> for Error {
    fn from(err: nom::Err<(&'a str, nom::error::ErrorKind)>) -> Self {
        use nom::Err as e;
        match err {
            e::Incomplete(_) => {
                unreachable!("nom::Err::Incomplete should only exist in streaming parsers")
            }
            e::Error((i, e)) => Error::Nom(i.into(), e),
            e::Failure((i, e)) => Error::Nom(i.into(), e),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self) // muahahahaha
    }
}

impl std::error::Error for Error {}

pub trait WrapMeta<T> {
    fn wrap_meta(self, s: &str) -> Result<T, Error>;
}

impl<T, E: Into<Error>> WrapMeta<T> for Result<T, E> {
    /// Wraps an Err in an OnLine(line_string, Err)
    fn wrap_meta(self, s: &str) -> Result<T, Error> {
        self.map_err(|e| Error::OnLine(s.to_owned(), Box::new(e.into())))
    }
}

struct LossyLines {
    reader: BufReader<File>,
    buf: Vec<u8>,
}

impl LossyLines {
    fn new(reader: BufReader<File>) -> Self {
        LossyLines {
            reader,
            buf: vec![],
        }
    }
}

// TODO: replace Strings in the parser iterators by a Cow
impl Iterator for LossyLines {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.buf.clear();
        let bytes_read = self.reader
            .read_until(b'\n', &mut self.buf)
            .expect("LossyLines reader shouldn't fail to read a line");

        if bytes_read == 0 {
            return None;
        }

        let line = String::from_utf8_lossy(&self.buf);
        let line = line.trim_end_matches("\r\n").trim_end_matches("\n");
        Some(line.to_owned())
    }
}

/// Returns an iterator over the lines of a file
pub fn file_lines<P: AsRef<Path>>(filepath: P) -> Result<impl Iterator<Item = String>, Error> {
    let reader = File::open(filepath).map(BufReader::new)?;
    Ok(LossyLines::new(reader))
}
