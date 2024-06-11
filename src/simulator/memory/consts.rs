pub const DATA_SIZE: usize = 0x0040_0000; // TODO: this, but I think it's about this much
pub const MMIO_SIZE: usize = 0x0022_0000;
pub const MMIO_START: usize = 0xff00_0000;

pub const HEAP_START: usize = 0x1004_0000;

pub use crate::renderer::{FRAME_0, FRAME_1, FRAME_SIZE, KDMMIO_CONTROL, KDMMIO_DATA};
pub const VIDEO_START: usize = MMIO_START + FRAME_0;
pub const VIDEO_END: usize = MMIO_START + FRAME_1 + FRAME_SIZE;

pub const TRANSPARENT_BYTE: u8 = 0xC7;
pub const TRANSPARENT_WORD: u32 = 0xC7C7C7C7;
