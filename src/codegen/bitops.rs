use std::ops::Range;

pub const fn mask(range: Range<u32>) -> u32 {
    ((1 << (range.end - range.start)) - 1) << range.start
}

pub trait BitOps: Sized {
    fn mask(self, ranger: Range<Self>) -> Self;
}

impl BitOps for u32 {
    fn mask(self, range: Range<Self>) -> Self {
        self & mask(range)
    }
}
