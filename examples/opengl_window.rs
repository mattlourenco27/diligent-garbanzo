use core::ffi::{c_void, CStr};

use drawsvg::sdl_wrapper::SDLContext;
use sdl2::event::Event;

const VERTEX_SHADER: &CStr = c"#version 150 core

in vec2 position;
in vec3 color;

out vec3 Color;

void main()
{
    Color = color;
    gl_Position = vec4(position, 0.0, 1.0);
}";

const FRAGMENT_SHADER: &CStr = c"#version 150 core

in vec3 Color;

out vec4 outColor;

void main()
{
    outColor = vec4(Color, 1.0);
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
    let vertices: [f32; 20] = [
        -0.5,  0.5, 1.0, 0.0, 0.0, // Top-left
        0.5,  0.5, 0.0, 1.0, 0.0, // Top-right
        0.5, -0.5, 0.0, 0.0, 1.0, // Bottom-right
        -0.5, -0.5, 1.0, 1.0, 1.0, // Bottom-left
    ];

    let mut ebo: gl::types::GLuint = 0; // Element buffer object index
    let elements: [gl::types::GLuint; 6] = [
        0, 1, 2,
        2, 3, 0
    ];

    unsafe {
        println!("Creating a vertex array object to save array settings to");

        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        println!("Creating array buffer");

        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            std::mem::size_of_val(&vertices) as gl::types::GLsizeiptr,
            vertices.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );

        println!("Creating element buffer");

        gl::GenBuffers(1, &mut ebo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            std::mem::size_of_val(&elements) as gl::types::GLsizeiptr,
            elements.as_ptr() as *const c_void,
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
        gl::BindFragDataLocation(shader_program, 0, c"outColor".as_ptr());

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

        let position_attribute = gl::GetAttribLocation(shader_program, c"position".as_ptr());

        gl::EnableVertexAttribArray(position_attribute as gl::types::GLuint);

        gl::VertexAttribPointer(
            position_attribute as gl::types::GLuint,
            2,
            gl::FLOAT,
            gl::FALSE,
            5 * std::mem::size_of::<f32>() as gl::types::GLsizei,
            std::ptr::null(),
        );

        let error = gl::GetError();
        if error != gl::NO_ERROR {
            println!("{error}");
        }

        let color_attribute = gl::GetAttribLocation(shader_program, c"color".as_ptr());

        gl::EnableVertexAttribArray(color_attribute as gl::types::GLuint);

        gl::VertexAttribPointer(
            color_attribute as gl::types::GLuint,
            3,
            gl::FLOAT,
            gl::FALSE,
            5 * std::mem::size_of::<f32>() as gl::types::GLsizei,
            (2 * std::mem::size_of::<f32>()) as *const c_void,
        );

        let error = gl::GetError();
        if error != gl::NO_ERROR {
            println!("{error}");
        }
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

            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
        }

        window.gl_swap_window();

        frames += 1;
    }

    println!("There were {} frames", frames);

    unsafe {
        gl::DeleteProgram(shader_program);
        gl::DeleteShader(fragment_shader);
        gl::DeleteShader(vertex_shader);

        gl::DeleteBuffers(1, &mut ebo);
        gl::DeleteBuffers(1, &mut vbo);

        gl::DeleteVertexArrays(1, &mut vao);
    }
}
