use std::ffi::OsString;
use std::{env, path::PathBuf};

use drawsvg::objects::ObjectMgr;
use drawsvg::render::CanvasRenderer;
use drawsvg::{objects::svg, sdl_wrapper::SDLContext};
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

    let mut object_mgr = ObjectMgr::new();
    object_mgr.add_object(svg_object.into());

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

    let mut renderer = match CanvasRenderer::new(window, &object_mgr) {
        Ok(renderer) => renderer,
        Err(err) => {
            println!("Error while building a renderer: {}", err);
            return;
        }
    };

    renderer.viewer.center_on_object(object_mgr.get_objects().get(0).unwrap());

    let mut frames = 0 as u32;

    'running: loop {
        for event in sdl_context.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        let keyboard_state = sdl_context.event_pump.keyboard_state();
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::I) {
            renderer.viewer.zoom_by(1.1);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::O) {
            renderer.viewer.zoom_by(1.0 / 1.1);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Left) {
            renderer.viewer.move_by([-10.0, 0.0].into());
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Up) {
            renderer.viewer.move_by([0.0, -10.0].into());
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Right) {
            renderer.viewer.move_by([10.0, 0.0].into());
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Down) {
            renderer.viewer.move_by([0.0, 10.0].into());
        }

        renderer.clear();
        renderer.render_objects();
        renderer.present();

        frames += 1;
    }

    println!("There were {} frames", frames);
}
