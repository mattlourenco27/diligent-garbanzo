use sdl2::{
    video::{GLContext, GLProfile, Window},
    EventPump, Sdl,
};

pub struct SDLContext {
    pub sdl: Sdl,
    pub window: Window,
    pub gl_context: GLContext,
    pub event_pump: EventPump,
}

impl SDLContext {
    pub fn new(width: u32, height: u32) -> Result<Self, String> {
        let sdl = sdl2::init()?;

        let video_subsystem = sdl.video()?;

        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 3);

        let window = match video_subsystem
            .window("My window", width, height)
            .opengl()
            .build()
        {
            Ok(window) => window,
            Err(error) => return Err(format!("{error:?}")),
        };

        let gl_context = window.gl_create_context()?;
        gl::load_with(|name| {
            video_subsystem.gl_get_proc_address(name) as *const std::os::raw::c_void
        });

        debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
        debug_assert_eq!(gl_attr.context_version(), (3, 3));

        let event_pump = sdl.event_pump()?;

        Ok(SDLContext {
            sdl,
            window,
            gl_context,
            event_pump,
        })
    }
}
