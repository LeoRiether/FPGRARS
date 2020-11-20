mod scancode;

use glium::glutin;
use pixel_canvas::{
    canvas::CanvasInfo,
    input::{Event, WindowEvent},
    Canvas, Color,
};
use std::sync::{Arc, Mutex};

use crate::simulator::MMIO;

pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 240;
pub const FRAME_SELECT: usize = 0x604;

struct MyState {
    mmio: Arc<MMIO>,
}

impl MyState {
    fn new(mmio: Arc<MMIO>) -> Self {
        Self { mmio }
    }

    fn handle_input(_info: &CanvasInfo, state: &mut MyState, event: &Event<()>) -> bool {
        match event {
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
                let mut mmio = state.mmio.sections[2].lock().unwrap();
                mmio[0] = 1;
                mmio[4] = scancode::to_ascii(*key);
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

pub fn init(mmio: Arc<MMIO>) {
    let canvas = Canvas::new(2 * WIDTH, 2 * HEIGHT)
        .title("FPGRARS")
        .state(MyState::new(mmio.clone()))
        .input(MyState::handle_input);

    #[cfg(feature = "show_ms")]
    let canvas = canvas.show_ms(true);

    canvas.render(move |_state, image| {
        let sec2 = mmio.sections[2].lock().unwrap();

        let frame = if sec2[FRAME_SELECT] == 0 { 0 } else { 1 };
        let mmio = mmio.sections[frame].lock().unwrap();

        // Draw each MMIO pixel as a 2x2 square
        for (y, row) in image.chunks_mut(2 * WIDTH).enumerate() {
            for (x, pixel) in row.iter_mut().enumerate() {
                let (x, y) = (x / 2, HEIGHT-1 - y / 2);
                let index = y * WIDTH + x;

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
