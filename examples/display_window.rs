use drawsvg::sdl_wrapper::SDLContext;
use sdl2::{event::Event, pixels::Color, rect::{Point, Rect}};

fn main() {
    let mut sdl_context = match SDLContext::new(800, 600) {
        Ok(sdl_context) => sdl_context,
        Err(string) => {
            println!("Error while setting up sdl context: {}", string);
            return;
        }
    };

    let mut frames = 0 as u32;
    let mut hue: i32 = 0;

    'running: loop {
        for event in sdl_context.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        let canvas = &mut sdl_context.canvas;
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        canvas.set_draw_color(Color::RGB(10, 50, hue as u8));
        canvas.fill_rect(Rect::new(10, 10, 780, 580)).unwrap();

        canvas.set_draw_color(Color::RGB(100, 0, 0));
        canvas.draw_point(Point::new(15, 15)).unwrap();

        canvas.present();

        frames += 1;
        hue = (hue + 1) % (core::u8::MAX as i32);
    }

    println!("There were {} frames", frames);
}
