use drawsvg::sdl_wrapper::ContextWindow;
use sdl2::event::Event;

fn main() {
    let mut context_window = ContextWindow::new(800, 600).unwrap();

    'running: loop {
        for event in context_window.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }
    }
}
