use std::{env, ffi::OsString, path::PathBuf};

use sdl2::event::Event;

use drawsvg::{
    objects::{svg, ObjectMgr},
    render::{canvas, gl},
    sdl_wrapper::SDLContext,
    tools::FpsCounter,
    vector::Vector2D,
};

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

    let mut renderer = match sdl_context.build_new_gl_window("My Window", 800, 100, &object_mgr) {
        Ok(renderer) => renderer,
        Err(err) => {
            println!("Error while building a new window: {}", err);
            return;
        }
    };

    renderer
        .get_viewer()
        .center_on_object(object_mgr.get_objects().get(0).unwrap());

    let mut frame_counter = FpsCounter::new();
    frame_counter.begin_measuring();

    'running: loop {
        for event in sdl_context.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        let keyboard_state = sdl_context.event_pump.keyboard_state();
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::I) {
            renderer.get_viewer().zoom_by(1.1);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::O) {
            renderer.get_viewer().zoom_by(1.0 / 1.1);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Left) {
            renderer.get_viewer().move_by(Vector2D::from([-10.0, 0.0]));
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Up) {
            renderer.get_viewer().move_by(Vector2D::from([0.0, -10.0]));
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Right) {
            renderer.get_viewer().move_by(Vector2D::from([10.0, 0.0]));
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Down) {
            renderer.get_viewer().move_by(Vector2D::from([0.0, 10.0]));
        }

        renderer.clear();
        renderer.render_objects();
        renderer.present();

        frame_counter.incr_frame_count();
    }
}
