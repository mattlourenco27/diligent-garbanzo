use drawsvg::sdl_wrapper::SDLContext;
use sdl2::event::Event;

fn main() {
    let mut sdl_context = match SDLContext::new(800, 600) {
        Ok(sdl_context) => sdl_context,
        Err(string) => {
            println!("Error while setting up sdl context: {}", string);
            return;
        }
    };

    'running: loop {
        for event in sdl_context.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        unsafe {
            gl::ClearColor(0.9, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        sdl_context.window.gl_swap_window();
    }
}
