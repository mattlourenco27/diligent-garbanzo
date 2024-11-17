use std::{
    env,
    ffi::OsString,
    path::PathBuf,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use sdl2::event::Event;

use drawsvg::{
    objects::{svg, ObjectMgr},
    render::CanvasRenderer,
    sdl_wrapper::SDLContext,
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

fn wait_for_duration(tx: mpsc::Sender<Duration>, duration: Duration) {
    thread::spawn(move || {
        let start_time = Instant::now();
        thread::sleep(duration);
        match tx.send(Instant::now().duration_since(start_time)) {
            Ok(_) => (),
            Err(_) => (),
        };
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

    renderer
        .viewer
        .center_on_object(object_mgr.get_objects().get(0).unwrap());

    let mut frames = 0 as u32;
    let (tx, rx) = mpsc::channel();
    wait_for_duration(tx.clone(), Duration::from_secs(5));

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
            renderer.viewer.move_by(Vector2D::from([-10.0, 0.0]));
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Up) {
            renderer.viewer.move_by(Vector2D::from([0.0, -10.0]));
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Right) {
            renderer.viewer.move_by(Vector2D::from([10.0, 0.0]));
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Down) {
            renderer.viewer.move_by(Vector2D::from([0.0, 10.0]));
        }

        renderer.clear();
        renderer.render_objects();
        renderer.present();

        frames += 1;

        match rx.try_recv() {
            Ok(time_elapsed) => {
                println!(
                    "Roughly 5 secs have passed. {} fps",
                    frames as f64 / time_elapsed.as_millis() as f64 * 1000.0
                );
                frames = 0;
                wait_for_duration(tx.clone(), Duration::from_secs(5));
            }
            Err(_) => (),
        }
    }
}
