mod scancode;

use glium::glutin;
use pixel_canvas::{
    canvas::CanvasInfo,
    input::{Event, WindowEvent},
    Canvas, Color,
};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

struct MyState {
    key_buffer: VecDeque<u8>,
}

impl MyState {
    fn new() -> Self {
        Self {
            key_buffer: VecDeque::new(),
        }
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
                dbg!(scancode::to_ascii(*key) as char);
                state.key_buffer.push_back(scancode::to_ascii(*key));
                true
            }

            _ => false,
        }
    }
}

fn mmio_color_to_rgb(x: u8) -> Color {
    let r = x & 0b111;
    let g = (x>>3) & 0b111;
    let b = x>>6;
    Color {
        r: r * 32,
        g: g * 32,
        b: b * 64,
    }
}

pub fn init(mmio: Arc<Mutex<Vec<u8>>>) {
    let canvas = Canvas::new(320, 240)
        .title("FPGRARS")
        .state(MyState::new())
        .input(MyState::handle_input);

    #[cfg(debug_assertions)]
    let canvas = canvas.show_ms(true);

    // The canvas will render for you at up to 60fps.
    canvas.render(move |_state, image| {
        // Modify the `image` based on your state.
        let mmio = mmio.lock().unwrap();

        for (i, pixel) in image.iter_mut().enumerate() {
            // let x = unsafe { *mmio.get_unchecked(i) };
            let x = mmio[i];
            if x != 0xC7 { // 0xC7 is "transparent"
                *pixel = mmio_color_to_rgb(x);
            }
        }
    });
}
