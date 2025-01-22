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
        -0.5, 0.5, 1.0, 0.0, 0.0, // Top-left
        0.5, 0.5, 0.0, 1.0, 0.0, // Top-right
        0.5, -0.5, 0.0, 0.0, 1.0, // Bottom-right
        -0.5, -0.5, 1.0, 1.0, 1.0, // Bottom-left
    ];

    let mut ebo: gl::types::GLuint = 0; // Element buffer object index
    let elements: [gl::types::GLuint; 6] = [0, 1, 2, 2, 3, 0];

    unsafe {
        println!("Creating a vertex array object to save array settings to");

        // A vertex array object will remember the data format of
        // 'position' and 'color' when we set them up later and also which vbo they get their data from.
        gl::GenVertexArrays(1, &mut vao); // Create vertex array object
        gl::BindVertexArray(vao); // Enable this vertex array object.

        println!("Creating array buffer");

        gl::GenBuffers(1, &mut vbo); // Create vertex buffer
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo); // Make active object (buffer)
        gl::BufferData(
            gl::ARRAY_BUFFER,
            std::mem::size_of_val(&vertices) as gl::types::GLsizeiptr,
            vertices.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        ); // Send data to the active buffer

        println!("Creating element buffer");

        gl::GenBuffers(1, &mut ebo); // Create element buffer
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo); // Make active object (element buffer)
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            std::mem::size_of_val(&elements) as gl::types::GLsizeiptr,
            elements.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        ); // Send data to the active element buffer

        println!("Compiling Vertex Shader");

        vertex_shader = gl::CreateShader(gl::VERTEX_SHADER); // Create shader
        gl::ShaderSource(
            vertex_shader,
            1,
            [VERTEX_SHADER].as_ptr() as *const *const gl::types::GLchar,
            std::ptr::null(),
        ); // Send a reference to the shader function

        gl::CompileShader(vertex_shader); // Compile the shader

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

        fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER); // Create shader
        gl::ShaderSource(
            fragment_shader,
            1,
            [FRAGMENT_SHADER].as_ptr() as *const *const gl::types::GLchar,
            std::ptr::null(),
        ); // Send a reference to the shader function

        gl::CompileShader(fragment_shader); // Compile the shader

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

        shader_program = gl::CreateProgram(); // Create a program
        gl::AttachShader(shader_program, vertex_shader); // Attach the vertex shader to the program
        gl::AttachShader(shader_program, fragment_shader); // Attach the fragment shader to the program

        let error = gl::GetError();
        if error != gl::NO_ERROR {
            println!("{error}");
        }

        println!("Binding the fragment shader to the right framebuffer");

        // Fragment shaders can have multiple outputs so we need to assign a framebuffer to each output.
        // Technically this is not needed because when there is only one output it is assigned to 0 by default.
        gl::BindFragDataLocation(shader_program, 0, c"outColor".as_ptr());

        let error = gl::GetError();
        if error != gl::NO_ERROR {
            println!("{error}");
        }

        println!("Linking the program");

        gl::LinkProgram(shader_program); // Load and link all shaders attached to this program

        let error = gl::GetError();
        if error != gl::NO_ERROR {
            println!("{error}");
        }

        println!("Activating the program");

        gl::UseProgram(shader_program); // Switch to using this program

        let error = gl::GetError();
        if error != gl::NO_ERROR {
            println!("{error}");
        }

        println!("Defining vertex attribute format");

        let position_attribute = gl::GetAttribLocation(shader_program, c"position".as_ptr()); // Get a reference to the 'position' input

        // 'position' contains 2 floats.
        // It does not need to be normalized.
        // The 'position' attributes are 5*sizeof(float) bytes apart.
        // The first float is at byte 0 (NULL).
        //
        // Calling this also saves the current vertex buffer object as the source.
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

        gl::EnableVertexAttribArray(position_attribute as gl::types::GLuint); // Enable the 'position' vertex attribute array

        let color_attribute = gl::GetAttribLocation(shader_program, c"color".as_ptr()); // Get a reference to the 'color' input

        // 'color contains 3 floats.
        // It does not need to be normalized.
        // The 'color' attributes are 5*sizeof(float) bytes apart.
        // The first float is at byte 2*sizeof(float).
        //
        // Calling this also saves the current vertex buffer object as the source.
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
        
        // Not calling this means that 'color' wont be processed
        gl::EnableVertexAttribArray(color_attribute as gl::types::GLuint); // Enable the 'color' vertex attribute array
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

            // Using the current ebo,
            // draw triangles.
            // Draw 6 indices total which are each of type unsigned int.
            // The first byte is at 0 (null)
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());

            // Using the current vao,
            // draw points.
            // The first one to draw is the 0th one
            // and draw 4 vertices total.
            // gl::DrawArrays(gl::POINTS, 0, 4);
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
