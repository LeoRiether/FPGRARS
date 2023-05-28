use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

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
        let bytes_read = self
            .reader
            .read_until(b'\n', &mut self.buf)
            .expect("LossyLines reader shouldn't fail to read a line");

        if bytes_read == 0 {
            return None;
        }

        let line = String::from_utf8_lossy(&self.buf);
        let line = line.trim_end_matches("\r\n").trim_end_matches('\n');
        Some(line.to_owned())
    }
}

/// Returns an iterator over the lines of a file
pub fn file_lines<P: AsRef<Path>>(filepath: P) -> Result<impl Iterator<Item = String>, io::Error> {
    let reader = File::open(filepath).map(BufReader::new)?;
    Ok(LossyLines::new(reader))
}
