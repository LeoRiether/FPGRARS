use glium::glutin;
use pixel_canvas::{
    canvas::CanvasInfo,
    input::{Event, WindowEvent},
    Canvas, Color,
};
use std::sync::{Arc, Mutex};

pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 240;
pub const FRAME_SELECT: usize = 0x20_0604;
pub const FRAME_0: usize = 0;
pub const FRAME_1: usize = 0x10_0000;
const KEYBOARD: usize = 0x20_0000;
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

struct MyState {
    mmio: Arc<Mutex<Vec<u8>>>,
}

impl MyState {
    fn new(mmio: Arc<Mutex<Vec<u8>>>) -> Self {
        Self { mmio }
    }

    fn handle_input(_info: &CanvasInfo, state: &mut MyState, event: &Event<()>) -> bool {
        match event {
            // Match a received character
            Event::WindowEvent {
                event: WindowEvent::ReceivedCharacter(chr),
                ..
            } => {
                let chr = if *chr == '\r' { '\n' } else { *chr };

                let mut mmio = state.mmio.lock().unwrap();

                mmio[KEYBOARD] = 1;
                mmio[KEYBOARD + 4] = chr as u8;

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
                let mut mmio = state.mmio.lock().unwrap();

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
                let mut mmio = state.mmio.lock().unwrap();

                push_key_to_buffer(&mut mmio, 0xF0);
                push_key_to_buffer(&mut mmio, *key as u8);

                remove_key_from_map(&mut mmio, *key as u8);

                true
            }

            _ => false,
        }
    }
}

// TODO: change the color format in pixel-canvas to ClientFormat::U8
fn mmio_color_to_rgb(x: u8) -> Color {
    let r = x & 0b111;
    let g = (x >> 3) & 0b111;
    let b = x >> 6;
    Color {
        r: r * 36,
        g: g * 36,
        b: b * 85,
    }
}

pub fn init(mmio: Arc<Mutex<Vec<u8>>>) {
    let canvas = Canvas::new(2 * WIDTH, 2 * HEIGHT)
        .title("FPGRARS")
        .state(MyState::new(mmio.clone()))
        .input(MyState::handle_input);

    #[cfg(feature = "show_ms")]
    let canvas = canvas.show_ms(true);

    canvas.render(move |_state, image| {
        let mmio = mmio.lock().unwrap();

        let frame = mmio[FRAME_SELECT];
        let start = if frame == 0 { FRAME_0 } else { FRAME_1 };

        // Draw each MMIO pixel as a 2x2 square
        for (y, row) in image.chunks_mut(2 * WIDTH).enumerate() {
            for (x, pixel) in row.iter_mut().enumerate() {
                let (x, y) = (x / 2, HEIGHT - 1 - y / 2);
                let index = start + y * WIDTH + x;

                let col = if cfg!(debug_assertions) {
                    *mmio
                        .get(index)
                        .expect("Out of bound access to the video memory!")
                } else {
                    unsafe { *mmio.get_unchecked(index) }
                };

                // if col != 0xc7 {
                *pixel = mmio_color_to_rgb(col);
                // }
            }
        }

        // Alternative, possibly slower, implementation:

        // let mut set = move |i, col| {
        //     if cfg!(debug_assertions) {
        //         *image
        //             .get_mut(i)
        //             .expect("Out of bounds access to the video memory!") = mmio_color_to_rgb(col);
        //     } else {
        //         unsafe {
        //             *image.get_unchecked_mut(i) = mmio_color_to_rgb(col);
        //         }
        //     }
        // };

        // for i in 0..FRAME_SIZE {
        //     let col = mmio[i + start];

        //     // 0xC7 is "transparent"
        //     if col != 0xC7 {
        //         // Don't ask
        //         // TODO: if this is too slow, we can try filling in line by line,
        //         // as every other line is just a copy of the one above it
        //         {
        //             set((i % WIDTH) * 2 + (i / WIDTH) * WIDTH * 4, col);
        //             set(1 + (i % WIDTH) * 2 + (i / WIDTH) * WIDTH * 4, col);
        //             set((i % WIDTH) * 2 + (i / WIDTH) * WIDTH * 4 + 2 * WIDTH, col);
        //             set(
        //                 1 + (i % WIDTH) * 2 + (i / WIDTH) * WIDTH * 4 + 2 * WIDTH,
        //                 col,
        //             );
        //         }
        //     }
        // }
    });
}
