use sdl2::{
    video::{Window, WindowBuildError},
    EventPump, Sdl, VideoSubsystem,
};

pub struct SDLContext {
    pub sdl: Sdl,
    pub video_subsystem: VideoSubsystem,
    pub event_pump: EventPump,
}

impl SDLContext {
    pub fn new() -> Result<Self, String> {
        let sdl = sdl2::init()?;

        Ok(SDLContext {
            video_subsystem: sdl.video()?,
            event_pump: sdl.event_pump()?,
            sdl,
        })
    }

    pub fn build_new_window(
        &self,
        title: &str,
        width: u32,
        height: u32,
    ) -> Result<Window, WindowBuildError> {
        self.video_subsystem.window(title, width, height).build()
    }
}
