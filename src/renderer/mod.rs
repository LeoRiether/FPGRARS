use glium::glutin;
use parking_lot::Mutex;
use pixel_canvas::{
    canvas::CanvasInfo,
    input::{Event, WindowEvent},
    Canvas, Color,
};
use std::sync::Arc;

pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 240;
pub const FRAME_SELECT: usize = 0x20_0604;
pub const FRAME_0: usize = 0;
pub const FRAME_1: usize = 0x10_0000;

/// Control bit for the Keyboard (Display?) MMIO.
/// `mmio[KDMMIO_CONTROL] == 1` means that a new key has been put in `mmio[KDMMIO_DATA]`, like a
/// keydown event. The control bit will be cleared right after you read a byte/half_word/word from
/// `mmio[KDMMIO_DATA]`. `KDMMIO_KEYDOWN` is easier to use, but not supported by other simulators.
pub const KDMMIO_CONTROL: usize = 0x20_0000;
pub const KDMMIO_DATA: usize = 0x20_0004;

/// `mmio[KDMMIO_KEYDOWN] == 1` means that some key is currently down/pressed. Not supported by
/// other simulators, but switching from `0x21` to `0x20` should be easy enough.
/// `mmio[KDMMIO_DATADOWN]` is a duplicate of `mmio[KDMMIO_DATA]`
pub const KDMMIO_KEYDOWN: usize = 0x21_0000;
pub const KDMMIO_DATADOWN: usize = 0x21_0004;

const KEYBUFFER: usize = 0x20_0100;
const KEYBUFFER_SIZE: usize = 8;
const KEYMAP: usize = 0x20_0520;

fn push_key_to_buffer(mmio: &mut [u8], key: u8) {
    // Shift buffer
    for i in (KEYBUFFER + 1..KEYBUFFER + KEYBUFFER_SIZE).rev() {
        mmio[i] = mmio[i - 1];
    }

    // Push key to mmio[KEYBUFFER]
    mmio[KEYBUFFER] = key;
}

fn push_key_to_map(mmio: &mut [u8], key: u8) {
    let (byte, bit) = (key / 8, key % 8);
    mmio[KEYMAP + byte as usize] |= 1 << bit;
}

fn remove_key_from_map(mmio: &mut [u8], key: u8) {
    let (byte, bit) = (key / 8, key % 8);
    mmio[KEYMAP + byte as usize] &= !(1 << bit);
}

struct InputState {
    mmio: Arc<Mutex<Vec<u8>>>,
}

impl InputState {
    fn new(mmio: Arc<Mutex<Vec<u8>>>) -> Self {
        Self { mmio }
    }

    fn handle_input(_info: &CanvasInfo, state: &mut InputState, event: &Event<()>) -> bool {
        match event {
            // Match a received character
            Event::WindowEvent {
                event: WindowEvent::ReceivedCharacter(chr),
                ..
            } => {
                let chr = if *chr == '\r' { '\n' } else { *chr };

                let mut mmio = state.mmio.lock();

                mmio[KDMMIO_CONTROL] = 1;
                mmio[KDMMIO_DATA] = chr as u8;

                mmio[KDMMIO_KEYDOWN] = 1;
                mmio[KDMMIO_DATADOWN] = chr as u8;

                true
            }

            // Match a keypress with scancode "key"
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            glutin::event::KeyboardInput {
                                state: glutin::event::ElementState::Pressed,
                                scancode: key,
                                ..
                            },
                        is_synthetic: false,
                        ..
                    },
                ..
            } => {
                let mut mmio = state.mmio.lock();

                push_key_to_buffer(&mut mmio, *key as u8);
                push_key_to_map(&mut mmio, *key as u8);

                true
            }

            // Match a keyup with scancode "key"
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            glutin::event::KeyboardInput {
                                state: glutin::event::ElementState::Released,
                                scancode: key,
                                ..
                            },
                        is_synthetic: false,
                        ..
                    },
                ..
            } => {
                let mut mmio = state.mmio.lock();

                mmio[KDMMIO_KEYDOWN] = 0;

                push_key_to_buffer(&mut mmio, 0xF0);
                push_key_to_buffer(&mut mmio, *key as u8);

                remove_key_from_map(&mut mmio, *key as u8);

                true
            }

            _ => false,
        }
    }
}

/// Provides the color that should be drawn at position (y, x) of the display
/// Basically a trait alias for Fn(memory, y, x) -> Color
/// The given `memory` slice starts at the beginning of the current frame
trait ColorProvider {
    fn get(&self, memory: &[u8], y: usize, x: usize) -> Color;
}

impl<F> ColorProvider for F
where
    F: Fn(&[u8], usize, usize) -> Color,
{
    fn get(&self, memory: &[u8], y: usize, x: usize) -> Color {
        self(memory, y, x)
    }
}

/// Opens a pixel-canvas window and draws from a given memory buffer.
/// The color provider is generally either
fn init_with_provider(mmio: Arc<Mutex<Vec<u8>>>, color_prov: impl ColorProvider + 'static) {
    let canvas = Canvas::new(2 * WIDTH, 2 * HEIGHT)
        .title("FPGRARS")
        .state(InputState::new(mmio.clone()))
        .input(InputState::handle_input);

    #[cfg(feature = "show_ms")]
    let canvas = canvas.show_ms(true);

    canvas.render(move |_state, image| {
        let mmio = mmio.lock();

        let frame = mmio[FRAME_SELECT];
        let start = if frame == 0 { FRAME_0 } else { FRAME_1 };

        // Draw each MMIO pixel as a 2x2 square
        for (y, row) in image.chunks_mut(2 * WIDTH).enumerate() {
            for (x, pixel) in row.iter_mut().enumerate() {
                *pixel = color_prov.get(&mmio[start..], y, x);
            }
        }
    });
}

/// Init the 8-bit (BBGGGRRR) format bitmap display
#[cfg(feature = "8-bit-display")]
pub fn init(mmio: Arc<Mutex<Vec<u8>>>) {
    let color_to_rgb = |x: u8| {
        let r = x & 0b111;
        let g = (x >> 3) & 0b111;
        let b = x >> 6;
        Color {
            r: r * 36,
            g: g * 36,
            b: b * 85,
        }
    };

    let color_provider = move |mmio: &[u8], y: usize, x: usize| {
        let (x, y) = (x / 2, HEIGHT - 1 - y / 2);
        let index = y * WIDTH + x;

        let x = if cfg!(debug_assertions) {
            *mmio
                .get(index)
                .expect("Out of bound access to the video memory!")
        } else {
            unsafe { *mmio.get_unchecked(index) }
        };

        color_to_rgb(x)
    };

    init_with_provider(mmio, color_provider);
}

/// Init the 24-bit (R8G8B8) format bitmap display
/// Note: this format is word-aligned, which means every color takes up
/// 32 bits in memory, but only 24 are actually used
#[cfg(not(feature = "8-bit-display"))]
pub fn init(mmio: Arc<Mutex<Vec<u8>>>) {
    let color_provider = |mmio: &[u8], y: usize, x: usize| {
        let bytes_per_pixel = 4;
        let (x, y) = (x / 2, HEIGHT - 1 - y / 2);
        let index = (y * WIDTH + x) * bytes_per_pixel;

        let get = |i| {
            if cfg!(debug_assertions) {
                *mmio
                    .get(i)
                    .expect("Out of bound access to the video memory!")
            } else {
                unsafe { *mmio.get_unchecked(i) }
            }
        };

        let (r, g, b) = (get(index + 2), get(index + 1), get(index));
        Color { r, g, b }
    };

    init_with_provider(mmio, color_provider);
}
