use byteorder::{ByteOrder, LittleEndian};
use std::io::Read;
use std::sync::{Arc, Mutex};

pub const DATA_SIZE: usize = 0x0040_0000; // TODO: this, but I think it's about this much
pub const MMIO_SIZE: usize = 0x0021_0000;
pub const MMIO_START: usize = 0xff00_0000;
pub const KBMMIO_CONTROL: usize = 0xff20_0000;
pub const KBMMIO_DATA: usize = 0xff20_0004;

pub const HEAP_START: usize = 0x1004_0000;

use crate::renderer::{FRAME_0, FRAME_1, HEIGHT, WIDTH};
pub const VIDEO_START: usize = MMIO_START + FRAME_0;
pub const VIDEO_END: usize = MMIO_START + FRAME_1 + WIDTH * HEIGHT;

const TRANSPARENT_BYTE: u8  = 0xC7;
const TRANSPARENT_WORD: u32 = 0xC7C7C7C7;

/// From https://graphics.stanford.edu/~seander/bithacks.html#ZeroInWord
/// I have no idea whether these u32 should be u32s or u64s because UL means nothing
#[inline]
fn has_zero_byte(v: u32) -> bool { (((v).wrapping_sub(0x01010101u32)) & !(v) & 0x80808080u32) != 0 }

#[inline]
fn has_transparent_byte(v: u32) -> bool { has_zero_byte(v ^ TRANSPARENT_WORD) }

/// Copies `n` bytes from `x` to the buffer, but ignores transparent bytes
fn copy_with_transparency(buf: &mut [u8], mut x: u32, n: usize) {
    for data in &mut buf[0..n] {
        let byte = x as u8;
        if byte != TRANSPARENT_BYTE {
            *data = byte;
        }
        x >>= 8;
    }
}

pub struct Memory {
    pub mmio: Arc<Mutex<Vec<u8>>>,
    pub data: Vec<u8>,

    /// Memory allocated by `sbrk`
    pub dynamic: Vec<u8>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            mmio: Arc::new(Mutex::new(vec![0; MMIO_SIZE])),
            data: vec![0; DATA_SIZE],
            dynamic: vec![],
        }
    }

    /// *IF* `x` has any transparent bytes and `i` is in the video memory,
    /// sets `n` bytes in the memory, ignoring the transparent ones. (`memory[i] = x`)
    /// Returns whether we actually set the bytes or not.
    fn set_with_transparency(&mut self, i: usize, x: u32, n: usize) -> bool {
        if has_transparent_byte(x) && (VIDEO_START..VIDEO_END).contains(&i) {
            let mut mmio = self.mmio.lock().unwrap();
            copy_with_transparency(&mut mmio[i - MMIO_START..], x, n);
            true
        } else {
            false
        }
    }

    /// Reads a value from the `i`-th byte of the memory
    pub fn get_with<T, F>(&self, i: usize, read: F) -> T
    where
        F: FnOnce(&[u8]) -> T,
    {
        if i >= MMIO_START {
            // MMIO
            let mut mmio = self.mmio.lock().unwrap();
            if i == KBMMIO_DATA {
                mmio[KBMMIO_CONTROL - MMIO_START] = 0;
            }
            read(&mmio[i - MMIO_START..])
        } else if i >= HEAP_START {
            // Heap/dynamic memory
            read(&self.dynamic[i - HEAP_START..])
        } else {
            // Data memory
            read(&self.data[i..])
        }
    }

    /// Writes the value `x` to the `i`-th byte of the memory, with some writing function `write`.
    /// Note: make sure `write` doesn't write 0xC7 (transparent) to video memory. In most cases,
    /// you should be using `set_byte`, `set_half` or `set_word` instead.
    fn set_with<T, F, R>(&mut self, i: usize, x: T, write: F) -> R
    where
        F: FnOnce(&mut [u8], T) -> R,
    {
        if i >= MMIO_START {
            // MMIO
            let mut mmio = self.mmio.lock().unwrap();
            write(&mut mmio[i - MMIO_START..], x)
        } else if i >= HEAP_START {
            // Heap/dynamic memory
            write(&mut self.dynamic[i - HEAP_START..], x)
        } else {
            // Data memory
            write(&mut self.data[i..], x)
        }
    }

    pub fn get_byte(&self, i: usize) -> u8 {
        self.get_with(i, |v| v[0])
    }

    pub fn set_byte(&mut self, i: usize, x: u8) {
        if self.set_with_transparency(i, x as u32, 1) { return; }
        self.set_with(i, x, |v, x| v[0] = x);
    }

    pub fn get_half(&self, i: usize) -> u16 {
        self.get_with(i, LittleEndian::read_u16)
    }

    pub fn set_half(&mut self, i: usize, x: u16) {
        if self.set_with_transparency(i, x as u32, 2) { return; }
        self.set_with(i, x, LittleEndian::write_u16);
    }

    pub fn get_word(&self, i: usize) -> u32 {
        self.get_with(i, LittleEndian::read_u32)
    }

    pub fn set_word(&mut self, i: usize, x: u32) {
        if self.set_with_transparency(i, x, 4) { return; }
        self.set_with(i, x, LittleEndian::write_u32);
    }

    pub fn get_float(&self, i: usize) -> f32 {
        self.get_with(i, LittleEndian::read_f32)
    }

    pub fn set_float(&mut self, i: usize, x: f32) {
        self.set_word(i, x.to_bits());
    }

    /// Tries to read `len` bytes from the reader and write them to `memory[start..start+len]`
    /// Returns the number of bytes read (or None if error)
    pub fn set_reader<R>(&mut self, reader: &mut R, start: usize, len: usize) -> Option<usize>
        where R: Read
    {
        // We'll write to these three sections separately
        let before_video = start..VIDEO_START.min(start+len);
        let in_video = VIDEO_START.max(start)..VIDEO_END.min(start+len);
        let after_video = VIDEO_END.max(start)..start+len;

        let mut bytes_read = 0;

        // Fast path: no need to check for transparent bytes
        if !before_video.is_empty() {
            let bytes = before_video.end - before_video.start;
            bytes_read += self.set_with(before_video.start, 0, |buf, _| {
                reader.take(bytes as u64).read(buf).ok()
            })?;
        }

        // Slow path: we need to check for transparent bytes and skip them
        if !in_video.is_empty() {
            let mut mmio = self.mmio.lock().unwrap();

            const MAX_LEN: usize = WIDTH * HEIGHT + 128; // most of the time `len` will be smaller
            let mut buf = vec![0; MAX_LEN.min(len)];

            let mut pos = in_video.start;
            while pos < in_video.end {
                // Read to the buffer. No more than `in_video.end - pos` bytes should be read,
                // or we'll read more than `len` bytes from the file
                let b = reader.take((in_video.end - pos) as u64).read(&mut buf).ok()?;
                bytes_read += b;

                // copy `b` bytes from `buf` to `mmio`
                let mut i = 0;
                while i+4 <= b {
                    let x = LittleEndian::read_u32(&buf[i..]);
                    // copy a word
                    if has_transparent_byte(x) {
                        copy_with_transparency(&mut mmio[pos + i - MMIO_START..], x, 4);
                    } else {
                        LittleEndian::write_u32(&mut mmio[pos + i - MMIO_START..], x);
                    }
                    i += 4;
                }
                while i+1 <= b {
                    // copy a byte
                    if !has_transparent_byte(buf[i] as u32) {
                        mmio[pos + i - MMIO_START] = buf[i];
                    }
                    i += 1;
                }

                pos += b;
            }
        }

        // Fast path: no need to check for transparent bytes
        if !after_video.is_empty() {
            let bytes = after_video.end - after_video.start;
            bytes_read += self.set_with(after_video.start, 0, |buf, _| {
                reader.take(bytes as u64).read(buf).ok()
            })?;
        }

        Some(bytes_read)
    }
}
