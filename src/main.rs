use std::{env, ffi::OsString, path::PathBuf, time::Instant};

use num_traits::Pow;
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::KeyboardState,
    mouse::{MouseState, MouseWheelDirection},
};

use drawsvg::{
    objects::{svg, ObjectMgr},
    render::Renderer,
    sdl_wrapper::SDLContext,
    tools::FpsCounter,
};

// Measured in pixels
const WINDOW_HEIGHT: u32 = 400;
const WINDOW_WIDTH: u32 = 800;

// Number of pixels to move per microsecond
const CAMERA_MOVE_SPEED: f32 = 0.0002;

// Fraction to zoom in or out by per microsecond.
// A value of 2.0 would double the zoom every microsecond exponentially.
// A value of 1.000001 works out to zooming by about 2.72x per second.
// (Don't you love it when things just work out to approximating 'e'?)
const KEYBOARD_ZOOM_IN_SPEED: f32 = 1.000001;
const KEYBOARD_ZOOM_OUT_SPEED: f32 = 1.0 / KEYBOARD_ZOOM_IN_SPEED;

// Fraction to zoom in or out by when the mouse wheel ticks up or down by one.
// A value of 1.1 increases the zoom by 1.1 per tick.
const MOUSE_ZOOM_IN_SPEED: f32 = 1.1;
const MOUSE_ZOOM_OUT_SPEED: f32 = 1.0 / MOUSE_ZOOM_IN_SPEED;

const WINDOW_TITLE: &str = "Diligent Garbanzo";

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

fn update_vsync_settings(
    sdl_context: &SDLContext,
    keyboard_state: &KeyboardState,
) -> Result<(), String> {
    let video_sys = &sdl_context.video_subsystem;
    if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::B) {
        video_sys.gl_set_swap_interval(sdl2::video::SwapInterval::Immediate)?;
    }
    if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::V) {
        video_sys.gl_set_swap_interval(sdl2::video::SwapInterval::VSync)?;
    }

    Ok(())
}

fn update_viewer_from_keyboard(
    renderer: &mut dyn Renderer,
    keyboard_state: &KeyboardState,
    us_of_frame: f32,
    object_mgr: &ObjectMgr,
) {
    let viewer = renderer.get_viewer();

    if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::I) {
        viewer.zoom_by(KEYBOARD_ZOOM_IN_SPEED.pow(us_of_frame));
    }
    if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::O) {
        viewer.zoom_by(KEYBOARD_ZOOM_OUT_SPEED.pow(us_of_frame));
    }
    if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::R) {
        if let Some(object) = object_mgr.get_objects().first() {
            viewer.center_on_object(object);
        }
    }
    if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Left) {
        viewer.move_by_pixels(-CAMERA_MOVE_SPEED * us_of_frame, 0.0);
    }
    if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Up) {
        viewer.move_by_pixels(0.0, -CAMERA_MOVE_SPEED * us_of_frame);
    }
    if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Right) {
        viewer.move_by_pixels(CAMERA_MOVE_SPEED * us_of_frame, 0.0);
    }
    if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Down) {
        viewer.move_by_pixels(0.0, CAMERA_MOVE_SPEED * us_of_frame);
    }
}

fn update_viewer_from_mouse_scrolling(renderer: &mut dyn Renderer, mouse_wheel_movement: f32) {
    if mouse_wheel_movement == 0.0 {
        return;
    }

    if mouse_wheel_movement > 0.0 {
        renderer
            .get_viewer()
            .zoom_by(MOUSE_ZOOM_IN_SPEED.pow(mouse_wheel_movement));
    } else {
        renderer
            .get_viewer()
            .zoom_by(MOUSE_ZOOM_OUT_SPEED.pow(-mouse_wheel_movement));
    }
}

fn update_viewer_from_mouse_position(
    renderer: &mut dyn Renderer,
    prev_state: &MouseState,
    curr_state: &MouseState,
) {
    if !prev_state.left() {
        return;
    }

    if curr_state.x() < 0
        || curr_state.x() as u32 >= renderer.width()
        || curr_state.y() < 0
        || curr_state.y() as u32 >= renderer.height()
    {
        return;
    }

    let delta_x = curr_state.x() - prev_state.x();
    let delta_y = curr_state.y() - prev_state.y();
    renderer
        .get_viewer()
        .move_by_pixels(-delta_x as f32, -delta_y as f32);
}

fn update_viewer_from_mouse(
    renderer: &mut dyn Renderer,
    prev_state: &MouseState,
    curr_state: &MouseState,
    mouse_wheel_movement: f32,
) {
    update_viewer_from_mouse_position(renderer, prev_state, curr_state);
    update_viewer_from_mouse_scrolling(renderer, mouse_wheel_movement);
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

    let mut renderer = match sdl_context.build_new_gl_window(
        WINDOW_TITLE,
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        &object_mgr,
    ) {
        Ok(renderer) => renderer,
        Err(err) => {
            println!("Error while building a new window: {}", err);
            return;
        }
    };

    renderer
        .get_viewer()
        .center_on_object(object_mgr.get_objects().get(0).unwrap());

    let mut last_mouse_state: Option<sdl2::mouse::MouseState> = None;

    let mut frame_counter = FpsCounter::new();
    frame_counter.begin_measuring();

    let mut frame_start_time = Instant::now();
    'running: loop {
        let mut mouse_wheel_movement: f32 = 0.0;

        for event in sdl_context.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::MouseWheel {
                    timestamp: _,
                    window_id: _,
                    which: _,
                    x: _,
                    y: _,
                    direction,
                    precise_x: _,
                    precise_y,
                    mouse_x: _,
                    mouse_y: _,
                } => {
                    mouse_wheel_movement = match direction {
                        MouseWheelDirection::Normal => precise_y,
                        MouseWheelDirection::Flipped => -precise_y,
                        MouseWheelDirection::Unknown(_) => 0.0,
                    }
                }
                Event::Window {
                    timestamp: _,
                    window_id: _,
                    win_event,
                } => match win_event {
                    WindowEvent::Resized(width, height)
                    | WindowEvent::SizeChanged(width, height) => {
                        if width < 0 || height < 0 {
                            continue;
                        }
                        renderer.resize_window(width as u32, height as u32);
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        renderer.clear();
        renderer.render_objects();
        renderer.present();

        let frame_end_time = Instant::now();
        let mut us_of_frame = frame_end_time.duration_since(frame_start_time).as_micros();
        if us_of_frame == 0 {
            us_of_frame = 1;
        }
        frame_start_time = frame_end_time;

        frame_counter.incr_frame_count();

        let keyboard_state = sdl_context.event_pump.keyboard_state();

        match update_vsync_settings(&sdl_context, &keyboard_state) {
            Err(err) => println!("Error while updating Vsync: {}", err),
            _ => (),
        };

        update_viewer_from_keyboard(
            renderer.as_mut(),
            &keyboard_state,
            us_of_frame as f32,
            &object_mgr,
        );

        let mouse_state = sdl_context.event_pump.mouse_state();
        if let Some(prev_state) = last_mouse_state {
            update_viewer_from_mouse(
                renderer.as_mut(),
                &prev_state,
                &mouse_state,
                mouse_wheel_movement,
            );
        }
        last_mouse_state = Some(mouse_state.clone());
    }
}
