pub struct Preprocessor<I>
where
    I: Iterator<Item = String>,
{
    items: I,
    buf: Vec<String>,
}

impl<I: Iterator<Item = String>> Iterator for Preprocessor<I> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(line) = self.buf.pop() {
            return Some(line);
        }

        let line = self.items.next()?;
        Some(line)
    }
}

/// Any iterator of strings can be preprocessed
pub trait Preprocess<I: Iterator<Item = String>> {
    fn preprocess(self) -> Preprocessor<I>
    where
        Self: Sized;
}

impl<I: Sized + Iterator<Item = String>> Preprocess<I> for I {
    fn preprocess(self) -> Preprocessor<I> {
        Preprocessor {
            items: self,
            buf: Vec::new(),
        }
    }
}
