use core::ffi::{c_void, CStr};

use drawsvg::sdl_wrapper::SDLContext;
use sdl2::event::Event;

const VERTEX_SHADER: &str = "#version 150 core

in vec2 position;

void main()
{
    gl_Position = vec4(position, 0.0, 1.0);
}";

const FRAGMENT_SHADER: &str = "#version 150 core

out vec4 outColor;

void main()
{
    outColor = vec4(1.0, 1.0, 1.0, 1.0);
}";

fn main() {
    let mut sdl_context = match SDLContext::new() {
        Ok(sdl_context) => sdl_context,
        Err(string) => {
            println!("Error while setting up sdl context: {}", string);
            return;
        }
    };

    let (window, _gl_ctx) = match sdl_context.build_new_gl_window("Example Window", 800, 600) {
        Ok(window) => window,
        Err(err) => {
            println!("Error while building a new OpenGL window: {}", err);
            return;
        }
    };

    let mut vertex_buffer_object_idx: gl::types::GLuint = 0;
    let vertices: [f32; 6] = [0.0, 0.5, 0.5, -0.5, -0.5, -0.5];
    unsafe {
        println!("Creating array buffer");

        gl::GenBuffers(1, &mut vertex_buffer_object_idx);
        gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer_object_idx);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            std::mem::size_of::<[f32; 6]>() as gl::types::GLsizeiptr,
            vertices.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );

        println!("Compiling Vertex Shader");

        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(
            vertex_shader,
            1,
            [VERTEX_SHADER].as_ptr() as *const *const gl::types::GLchar,
            [VERTEX_SHADER.len() as gl::types::GLint].as_ptr(),
        );

        gl::CompileShader(vertex_shader);

        let mut compile_status: gl::types::GLint = 0;
        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut compile_status);

        if compile_status == gl::TRUE as gl::types::GLint {
            println!("Shader compiled successfully")
        } else {
            let mut buffer = [0 as gl::types::GLchar; 512];
            gl::GetShaderInfoLog(
                vertex_shader,
                512,
                std::ptr::null_mut(),
                buffer.as_mut_ptr(),
            );
            println!("{:?}", CStr::from_ptr(buffer.as_ptr()));
            return;
        }

        println!("Compiling Fragment Shader");

        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(
            fragment_shader,
            1,
            [FRAGMENT_SHADER].as_ptr() as *const *const gl::types::GLchar,
            [FRAGMENT_SHADER.len() as gl::types::GLint].as_ptr(),
        );

        gl::CompileShader(fragment_shader);

        let mut compile_status: gl::types::GLint = 0;
        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut compile_status);

        if compile_status == gl::TRUE as gl::types::GLint {
            println!("Shader compiled successfully")
        } else {
            let mut buffer = [0 as gl::types::GLchar; 512];
            gl::GetShaderInfoLog(
                fragment_shader,
                512,
                std::ptr::null_mut(),
                buffer.as_mut_ptr(),
            );
            println!("{:?}", CStr::from_ptr(buffer.as_ptr()));
            return;
        }

        println!("Compiling shaders into a program");

        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);

        println!("Binding the fragment shader to the right framebuffer");

        gl::BindFragDataLocation(
            shader_program,
            0,
            "outColor".as_ptr() as *const gl::types::GLchar,
        );
    }

    let mut frames = 0 as u32;

    'running: loop {
        for event in sdl_context.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        unsafe {
            gl::ClearColor(0.6, 0.0, 0.8, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        window.gl_swap_window();

        frames += 1;
    }

    println!("There were {} frames", frames);
}
