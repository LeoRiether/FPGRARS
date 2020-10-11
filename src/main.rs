use glfw::{Action, Context as _, Key, WindowEvent};
use luminance_glfw::GlfwSurface;
use luminance_windowing::{WindowDim, WindowOpt};
use std::process::exit;

fn main() {
    // our graphics surface
    let dim = WindowDim::Windowed {
        width: 960,
        height: 540,
    };
    let surface = GlfwSurface::new_gl33("Hello, world!", WindowOpt::default().set_dim(dim));

    match surface {
        Ok(surface) => {
            eprintln!("graphics surface created");
            main_loop(surface);
        }

        Err(e) => {
            eprintln!("cannot create graphics surface:\n{}", e);
            exit(1);
        }
    }
}

fn main_loop(mut surface: GlfwSurface) {
    let start_t = Instant::now();
    let back_buffer = surface.back_buffer().unwrap();

    'app: loop {
        // handle events
        surface.window.glfw.poll_events();
        for (_, event) in surface.events_rx.try_iter() {
            match event {
                WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => {
                    break 'app
                }
                _ => (),
            }
        }

        // get the current time and create a color based on the time
        let t = start_t.elapsed().as_millis() as f32 * 1e-3;
        let color = [t.cos(), t.sin(), 0.5, 1.];

        let render = surface
            .new_pipeline_gate()
            .pipeline(
                &back_buffer,
                &PipelineState::default().set_clear_color(color),
                |_, _| Ok(()),
            )
            .assume();

        // swap buffer chains
        if render.is_ok() {
            surface.window.swap_buffers();
        } else {
            break 'app;
        }

        // swap buffer chains
        surface.window.swap_buffers();
    }
}
