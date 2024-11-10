use std::ffi::OsString;
use std::{env, path::PathBuf};

use drawsvg::render::CanvasRenderer;
use drawsvg::{sdl_wrapper::SDLContext, objects::svg};
use sdl2::event::Event;

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

    let mut sdl_context = match SDLContext::new() {
        Ok(sdl_context) => sdl_context,
        Err(string) => {
            println!("Error while setting up sdl context: {}", string);
            return;
        }
    };

    let window = match sdl_context.build_new_window("My Window", 800, 600) {
        Ok(window) => window,
        Err(err) => {
            println!("Error while building a new window: {}", err);
            return;
        }
    };

    let mut renderer = match CanvasRenderer::new(window) {
        Ok(renderer) => renderer,
        Err(err) => {
            println!("Error while building a renderer: {}", err);
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

        renderer.clear();
        renderer.render_svg(&svg_object);
        renderer.present();

        frames += 1;
    }

    println!("There were {} frames", frames);
}
