use std::io::{self, BufRead};

pub struct Utf8LossyLines<R: BufRead> {
    reader: R,
    buf: Vec<u8>,
}

impl<R: BufRead> Iterator for Utf8LossyLines<R> {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.buf.clear();
        match self.reader.read_until(b'\n', &mut self.buf) {
            Ok(0) => None,
            Ok(_) => Some(Ok(
                String::from_utf8_lossy(&self.buf[..self.buf.len() - 1]).into_owned()
            )),
            Err(e) => Some(Err(e)),
        }
    }
}

pub trait Utf8LossyLinesExt: BufRead + Sized {
    fn utf8_lossy_lines(self) -> Utf8LossyLines<Self>;
}

impl<R: BufRead> Utf8LossyLinesExt for R {
    fn utf8_lossy_lines(self) -> Utf8LossyLines<R> {
        Utf8LossyLines {
            reader: self,
            buf: Vec::new(),
        }
    }
}
