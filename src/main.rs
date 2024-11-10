use std::ffi::OsString;
use std::{env, path::PathBuf};

use drawsvg::render::CanvasRenderer;
use drawsvg::{sdl_wrapper::SDLContext, svg};
use sdl2::{event::Event, pixels::Color};

struct Args {
    svg_path: PathBuf,
}

fn print_usage() {
    println!("Usage: {} <svg file>", env!("CARGO_PKG_NAME"));
}

fn parse_args() -> Option<Args> {
    let args: Vec<OsString> = env::args_os().collect();
    if args.len() < 2 {
        print_usage();
        return None;
    }

    return Some(Args {
        svg_path: PathBuf::from(args.into_iter().nth(1).unwrap()),
    });
}

fn main() {
    let args = match parse_args() {
        None => return,
        Some(args) => args,
    };

    let svg_object = match svg::read_from_file(args.svg_path.as_ref()) {
        Err(err) => {
            println!("{}", err);
            return;
        }
        Ok(svg) => svg,
    };

    let mut sdl_context = match SDLContext::new(800, 600) {
        Ok(sdl_context) => sdl_context,
        Err(string) => {
            println!("Error while setting up sdl context: {}", string);
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

        let canvas = &mut sdl_context.canvas;
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        {
            canvas.set_draw_color(Color::RGB(50, 50, 50));

            let mut renderer = CanvasRenderer::new(canvas);
            renderer.render_svg(&svg_object);
        }

        canvas.present();

        frames += 1;
    }

    println!("There were {} frames", frames);
}
