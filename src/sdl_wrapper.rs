use sdl2::{video::Window, EventPump, Sdl};

pub struct ContextWindow {
    pub sdl: Sdl,
    pub window: Window,
    pub event_pump: EventPump,
}

impl ContextWindow {
    pub fn new(width: usize, height: usize) -> Result<Self, String> {
        let sdl = sdl2::init()?;

        let video_subsystem = sdl.video()?;

        let window = match video_subsystem
            .window("My window", width as u32, height as u32)
            .build()
        {
            Ok(window) => window,
            Err(error) => return Err(format!("{error:?}")),
        };

        let event_pump = sdl.event_pump()?;

        Ok(ContextWindow {
            sdl,
            window,
            event_pump,
        })
    }
}
