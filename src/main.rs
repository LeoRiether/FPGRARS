use std::collections::VecDeque;

use pixel_canvas::{
    canvas::CanvasInfo,
    input::{Event, WindowEvent},
    Canvas, Color,
};

use glium::glutin;

struct MyState {
    key_buffer: VecDeque<u8>,
}

impl MyState {
    fn new() -> Self {
        Self {
            key_buffer: VecDeque::new(),
        }
    }

    fn handle_input(info: &CanvasInfo, state: &mut MyState, event: &Event<()>) -> bool {
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
                dbg!(&key);
                // state.key_buffer.push_back(
                true
            }

            _ => false
        }
    }
}

fn main() {
    let canvas = Canvas::new(512, 512)
        .title("FPGRARS")
        .state(MyState::new())
        .input(MyState::handle_input)
        .render_on_change(true);

    #[cfg(debug_assertions)]
    let canvas = canvas.show_ms(true);

    // The canvas will render for you at up to 60fps.
    canvas.render(|state, image| {
        // Modify the `image` based on your state.
        let width = image.width() as usize;
        for (y, row) in image.chunks_mut(width).enumerate() {
            for (x, pixel) in row.iter_mut().enumerate() {
                *pixel = Color {
                    r: (x * y) as u8,
                    g: 0,
                    b: (x * y) as u8,
                }
            }
        }
    });
}
