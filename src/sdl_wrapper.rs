use sdl2::{render::Canvas, video::Window, EventPump, Sdl};

pub struct SDLContext {
    pub sdl: Sdl,
    pub canvas: Canvas<Window>,
    pub event_pump: EventPump,
}

impl SDLContext {
    pub fn new(width: u32, height: u32) -> Result<Self, String> {
        let sdl = sdl2::init()?;

        let video_subsystem = sdl.video()?;

        let window = match video_subsystem.window("My window", width, height).build() {
            Ok(window) => window,
            Err(error) => return Err(format!("{error:?}")),
        };

        let canvas = match window.into_canvas().present_vsync().build() {
            Ok(canvas) => canvas,
            Err(error) => return Err(format!("{error:?}")),
        };

        let event_pump = sdl.event_pump()?;

        Ok(SDLContext {
            sdl,
            canvas,
            event_pump,
        })
    }
}
