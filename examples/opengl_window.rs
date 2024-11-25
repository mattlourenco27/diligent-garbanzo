use core::ffi::{c_void, CStr};

use drawsvg::sdl_wrapper::SDLContext;
use sdl2::event::Event;

const VERTEX_SHADER: &CStr = c"#version 150 core

in vec2 position;

void main()
{
    gl_Position = vec4(position, 0.0, 1.0);
}";

const FRAGMENT_SHADER: &CStr = c"#version 150 core

uniform vec3 triangleColor;

out vec4 outColor;

void main()
{
    outColor = vec4(triangleColor, 1.0);
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

    let shader_program: gl::types::GLuint;
    let vertex_shader: gl::types::GLuint;
    let fragment_shader: gl::types::GLuint;
    let mut vao: gl::types::GLuint = 0; // Vertex array object index
    let mut vbo: gl::types::GLuint = 0; // Vertex buffer object index
    let vertices: [f32; 6] = [0.0, 0.5, 0.5, -0.5, -0.5, -0.5];

    let uni_color: gl::types::GLint;

    unsafe {
        println!("Creating a vertex array object to save array settings to");

        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        println!("Creating array buffer");

        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            std::mem::size_of::<[f32; 6]>() as gl::types::GLsizeiptr,
            vertices.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );

        println!("Compiling Vertex Shader");

        vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(
            vertex_shader,
            1,
            [VERTEX_SHADER].as_ptr() as *const *const gl::types::GLchar,
            std::ptr::null(),
        );

        gl::CompileShader(vertex_shader);

        let mut compile_status: gl::types::GLint = 0;
        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut compile_status);

        if compile_status == gl::TRUE as gl::types::GLint {
            println!("Shader compiled successfully");
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

        fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(
            fragment_shader,
            1,
            [FRAGMENT_SHADER].as_ptr() as *const *const gl::types::GLchar,
            std::ptr::null(),
        );

        gl::CompileShader(fragment_shader);

        let mut compile_status: gl::types::GLint = 0;
        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut compile_status);

        if compile_status == gl::TRUE as gl::types::GLint {
            println!("Shader compiled successfully");
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

        shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);

        let error = gl::GetError();
        if error != gl::NO_ERROR {
            println!("{error}");
        }

        println!("Binding the fragment shader to the right framebuffer");

        // Technically this is not needed because it is 0 by default.
        gl::BindFragDataLocation(
            shader_program,
            0,
            c"outColor".as_ptr(),
        );

        let error = gl::GetError();
        if error != gl::NO_ERROR {
            println!("{error}");
        }

        println!("Linking the program");

        gl::LinkProgram(shader_program);

        let error = gl::GetError();
        if error != gl::NO_ERROR {
            println!("{error}");
        }

        println!("Activating the program");

        gl::UseProgram(shader_program);

        let error = gl::GetError();
        if error != gl::NO_ERROR {
            println!("{error}");
        }

        println!("Defining vertex attribute format");

        let position_attribute = gl::GetAttribLocation(
            shader_program,
            c"position".as_ptr(),
        );

        let error = gl::GetError();
        if error != gl::NO_ERROR {
            println!("Error while getting attribute index of 'position': {error}");
        }

        println!("Attribute location is {position_attribute}");

        gl::VertexAttribPointer(
            position_attribute as gl::types::GLuint,
            2,
            gl::FLOAT,
            gl::FALSE,
            0,
            std::ptr::null(),
        );

        let error = gl::GetError();
        if error != gl::NO_ERROR {
            println!("Error while assigning attributes to 'position': {error}");
        }

        gl::EnableVertexAttribArray(position_attribute as gl::types::GLuint);

        let error = gl::GetError();
        if error != gl::NO_ERROR {
            println!("Error while enabling attributes of 'position': {error}");
        }

        println!("Retrieving the index of the uniform 'triangleColor'");

        uni_color = gl::GetUniformLocation(shader_program, c"triangleColor".as_ptr());
    }

    let mut frames = 0 as u32;
    let start_time = std::time::Instant::now();

    'running: loop {
        for event in sdl_context.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        let elapsed_time = std::time::Instant::now().duration_since(start_time);
        let red_value = ((elapsed_time.as_millis() as f32 * 0.005).sin() + 1.0) / 2.0;

        unsafe {
            gl::Uniform3f(uni_color, red_value, 0.0, 0.0);

            gl::ClearColor(0.6, 0.0, 0.8, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        window.gl_swap_window();

        frames += 1;
    }

    println!("There were {} frames", frames);

    unsafe {
        gl::DeleteProgram(shader_program);
        gl::DeleteShader(fragment_shader);
        gl::DeleteShader(vertex_shader);

        gl::DeleteBuffers(1, &mut vbo);

        gl::DeleteVertexArrays(1, &mut vao);
    }
}
