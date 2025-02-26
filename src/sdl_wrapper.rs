use sdl2::{
    EventPump, Sdl, VideoSubsystem,
};

use crate::{
    objects::ObjectMgr,
    render::{canvas::CanvasRenderer, gl::GLRenderer, Renderer},
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

    pub fn build_new_window<'a>(
        &self,
        title: &str,
        width: u32,
        height: u32,
        object_mgr: &'a ObjectMgr,
    ) -> Result<Box<dyn Renderer + 'a>, String> {
        let window = match self.video_subsystem.window(title, width, height).build() {
            Ok(window) => window,
            Err(err) => return Err(format!("{err}")),
        };

        match CanvasRenderer::<'a>::new(window, &object_mgr) {
            Ok(renderer) => Ok(Box::new(renderer)),
            Err(err) => Err(format!("{err}")),
        }
    }

    pub fn build_new_gl_window(
        &self,
        title: &str,
        width: u32,
        height: u32,
        object_mgr: &ObjectMgr,
    ) -> Result<Box<dyn Renderer>, String> {
        let gl_attr = self.video_subsystem.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(3, 3);

        let window = match self
            .video_subsystem
            .window(title, width, height)
            .opengl()
            .build()
        {
            Ok(window) => window,
            Err(err) => return Err(format!("{err}")),
        };

        let gl_ctx = window.gl_create_context().unwrap();
        gl::load_with(|name| self.video_subsystem.gl_get_proc_address(name) as *const _);

        let gl_attr = self.video_subsystem.gl_attr();
        debug_assert_eq!(gl_attr.context_profile(), sdl2::video::GLProfile::Core);
        debug_assert_eq!(gl_attr.context_version(), (3, 3));

        Ok(Box::new(GLRenderer::new(window, gl_ctx, &object_mgr)?))
    }
}
