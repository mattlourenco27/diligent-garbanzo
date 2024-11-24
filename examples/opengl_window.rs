use drawsvg::sdl_wrapper::SDLContext;
use sdl2::event::Event;

fn main() {
    let mut sdl_context = match SDLContext::new() {
        Ok(sdl_context) => sdl_context,
        Err(string) => {
            println!("Error while setting up sdl context: {}", string);
            return;
        }
    };

    let (window, _gl_ctx) = match sdl_context.build_new_gl_window("Example Window", 800, 600) {
        Ok(window) => window,
        Err(err) => {
            println!("Error while building a new OpenGL window: {}", err);
            return;
        }
    };

    let mut frames = 0 as u32;

    'running: loop {
        for event in sdl_context.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        unsafe {
            gl::ClearColor(0.6, 0.0, 0.8, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        window.gl_swap_window();

        frames += 1;
    }

    println!("There were {} frames", frames);
}
