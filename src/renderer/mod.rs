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

pub fn init(mmio: Arc<Mutex<Vec<u8>>>) {
    let canvas = Canvas::new(640, 480)
        .title("FPGRARS")
        .state(MyState::new())
        .input(MyState::handle_input);

    #[cfg(debug_assertions)]
    let canvas = canvas.show_ms(true);

    use std::thread;
    for delay in 1..=5 {
        let mmio = mmio.clone();
        thread::spawn(move || {
            thread::sleep(std::time::Duration::from_secs(delay));
            let mut mmio = mmio.lock().unwrap();
            for pixel in mmio.iter_mut().take(640 * 480) {
                *pixel = (delay * 255 / 5) as u8;
            }
        });
    }

    // The canvas will render for you at up to 60fps.
    canvas.render(move |_state, image| {
        // Modify the `image` based on your state.
        let mmio = mmio.lock().unwrap();

        for (i, pixel) in image.iter_mut().enumerate() {
            // let x = unsafe { *mmio.get_unchecked(i) };
            let x = mmio[i];
            *pixel = Color { r: x, g: x, b: x }
        }
    });
}
