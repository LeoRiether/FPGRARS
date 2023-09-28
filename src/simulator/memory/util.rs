use crate::simulator::{TRANSPARENT_BYTE, TRANSPARENT_WORD};

/// From https://graphics.stanford.edu/~seander/bithacks.html#ZeroInWord
/// I have no idea whether these u32 should be u32s or u64s because UL means nothing
#[inline]
fn has_zero_byte(v: u32) -> bool {
    (((v).wrapping_sub(0x01010101u32)) & !(v) & 0x80808080u32) != 0
}

#[inline]
pub fn has_transparent_byte(v: u32) -> bool {
    has_zero_byte(v ^ TRANSPARENT_WORD)
}

/// Copies `n` bytes from `x` to the buffer, but ignores transparent bytes
pub fn copy_with_transparency(buf: &mut [u8], mut x: u32, n: usize) {
    for data in &mut buf[0..n] {
        let byte = x as u8;
        if byte != TRANSPARENT_BYTE {
            *data = byte;
        }
        x >>= 8;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_has_zero_byte() {
        assert!(has_zero_byte(0xff00ff00));
        assert!(!has_zero_byte(0x10203040));
        assert!(has_zero_byte(0x0011ff22));
        assert!(!has_zero_byte(0x12345678));
        assert!(has_zero_byte(0x12005678));
        assert!(has_zero_byte(0x11223300));
    }
}
