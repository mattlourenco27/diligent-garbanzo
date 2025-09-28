use core::ffi::{c_void, CStr};
use gl::types::{GLchar, GLenum, GLint, GLsizei, GLuint};

use crate::matrix::Matrix3x3;

pub const POS_SIZE: u8 = 2;
pub const COLOR_SIZE: u8 = 4;

unsafe fn maybe_get_gl_error() -> Result<(), String> {
    let error = gl::GetError();
    if error != gl::NO_ERROR {
        return Err(format!("Error code: {error}"));
    }

    Ok(())
}

unsafe fn maybe_get_shader_compile_error(shader_id: GLuint) -> Result<(), String> {
    let mut compile_status: GLint = 0;
    gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, &mut compile_status);

    if compile_status != gl::TRUE as GLint {
        const BUF_SIZE: i32 = 512;
        let mut buffer = [0 as GLchar; BUF_SIZE as usize];
        gl::GetShaderInfoLog(
            shader_id,
            BUF_SIZE,
            std::ptr::null_mut(),
            buffer.as_mut_ptr(),
        );
        let c_slice = CStr::from_ptr(buffer.as_ptr());
        return Err(c_slice.to_string_lossy().into_owned());
    }

    Ok(())
}

unsafe fn send_and_compile_shader(
    shader_type: GLenum,
    shader_source: &CStr,
) -> Result<GLuint, String> {
    let shader_id = gl::CreateShader(shader_type);

    gl::ShaderSource(
        shader_id,
        1,
        [shader_source].as_ptr() as *const *const GLchar,
        std::ptr::null(),
    );

    gl::CompileShader(shader_id);

    maybe_get_shader_compile_error(shader_id)?;

    Ok(shader_id)
}

pub trait ShaderProgram {
    fn build() -> Result<Self, String>
    where
        Self: Sized;

    unsafe fn bind_attributes_to_vertex_array(&self);
    unsafe fn activate(&self);
}

pub struct LineShader {
    vertex_shader: GLuint,
    fragment_shader: GLuint,
    geometry_shader: GLuint,
    shader_program: GLuint,
    position_attr: GLuint,
    color_attr: GLuint,
    norm_to_viewer_uniform: GLuint,
    line_thickness_uniform: GLuint,
}

impl LineShader {
    const VERTEX_SHADER: &CStr = c"#version 150 core

in vec2 position;
in vec4 color;

out vec4 VertexColor;

void main() {
    VertexColor = color;
    gl_Position = vec4(position, 0.0, 1.0);
}";

    const FRAGMENT_SHADER: &CStr = c"#version 150 core

in vec4 GeoColor;

out vec4 outColor;

void main()
{
    outColor = GeoColor;
}";

    const GEOMETRY_SHADER: &CStr = c"#version 150 core
layout(lines) in;
layout(triangle_strip, max_vertices = 4) out;

in vec4 VertexColor[];
out vec4 GeoColor;

uniform float thickness;
uniform mat3 norm_to_viewer;

void main() {
    vec2 p0 = gl_in[0].gl_Position.xy;
    vec2 p1 = gl_in[1].gl_Position.xy;

    vec2 dir = normalize(p1 - p0);
    vec2 offset = vec2(-dir.y, dir.x) * thickness * 0.5;

    // Generate corners of rectangle
    vec2 v0 = p0 + offset;
    vec2 v1 = p0 - offset;
    vec2 v2 = p1 + offset;
    vec2 v3 = p1 - offset;

    // Transform to screen space when outputting
    GeoColor = VertexColor[0];
    vec3 transformed = vec3(v0, 1.0) * norm_to_viewer;
    gl_Position = vec4(transformed.x, -transformed.y, 0.0, 1.0);
    EmitVertex();
    
    transformed = vec3(v1, 1.0) * norm_to_viewer;
    gl_Position = vec4(transformed.x, -transformed.y, 0.0, 1.0);
    EmitVertex();
    
    GeoColor = VertexColor[1];
    transformed = vec3(v2, 1.0) * norm_to_viewer;
    gl_Position = vec4(transformed.x, -transformed.y, 0.0, 1.0);
    EmitVertex();
    
    transformed = vec3(v3, 1.0) * norm_to_viewer;
    gl_Position = vec4(transformed.x, -transformed.y, 0.0, 1.0);
    EmitVertex();
    
    EndPrimitive();
}";
}

impl LineShader {
    unsafe fn create_new_program(
        vertex_shader: GLuint,
        fragment_shader: GLuint,
        geometry_shader: GLuint,
    ) -> Result<GLuint, String> {
        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::AttachShader(shader_program, geometry_shader);

        maybe_get_gl_error()?;

        Ok(shader_program)
    }

    unsafe fn bind_fragment_shader_output(shader_program: GLuint) -> Result<(), String> {
        gl::BindFragDataLocation(shader_program, 0, c"outColor".as_ptr());

        maybe_get_gl_error()?;

        Ok(())
    }

    unsafe fn link_program(shader_program: GLuint) -> Result<(), String> {
        gl::LinkProgram(shader_program);

        maybe_get_gl_error()?;

        Ok(())
    }

    unsafe fn get_attributes(
        shader_program: GLuint,
    ) -> Result<(GLuint, GLuint, GLuint, GLuint), String> {
        let position_attribute = gl::GetAttribLocation(shader_program, c"position".as_ptr());

        maybe_get_gl_error()?;

        let color_attribute = gl::GetAttribLocation(shader_program, c"color".as_ptr());

        maybe_get_gl_error()?;

        let norm_to_viewer_uniform =
            gl::GetUniformLocation(shader_program, c"norm_to_viewer".as_ptr());

        let line_thickness_uniform = gl::GetUniformLocation(shader_program, c"thickness".as_ptr());

        maybe_get_gl_error()?;

        Ok((
            position_attribute as GLuint,
            color_attribute as GLuint,
            norm_to_viewer_uniform as GLuint,
            line_thickness_uniform as GLuint,
        ))
    }

    pub fn update_norm_to_viewer(&self, norm_to_viewer_transform: &Matrix3x3<f32>) {
        unsafe {
            gl::UniformMatrix3fv(
                self.norm_to_viewer_uniform as GLint,
                1,
                gl::TRUE,
                <&Matrix3x3<f32> as Into<&[[f32; 3]; 3]>>::into(norm_to_viewer_transform)[0]
                    .as_ptr(),
            );
        }
    }

    pub fn update_line_thickness(&self, thickness: f32) {
        unsafe {
            gl::Uniform1f(self.line_thickness_uniform as GLint, thickness);
        }
    }
}

impl ShaderProgram for LineShader {
    fn build() -> Result<LineShader, String> {
        unsafe {
            let vertex_shader =
                send_and_compile_shader(gl::VERTEX_SHADER, LineShader::VERTEX_SHADER)?;
            let fragment_shader =
                send_and_compile_shader(gl::FRAGMENT_SHADER, LineShader::FRAGMENT_SHADER)?;
            let geometry_shader =
                send_and_compile_shader(gl::GEOMETRY_SHADER, LineShader::GEOMETRY_SHADER)?;

            let shader_program =
                LineShader::create_new_program(vertex_shader, fragment_shader, geometry_shader)?;

            LineShader::bind_fragment_shader_output(shader_program)?;

            LineShader::link_program(shader_program)?;

            let (position_attr, color_attr, norm_to_viewer_uniform, line_thickness_uniform) =
                LineShader::get_attributes(shader_program)?;

            let shader = LineShader {
                vertex_shader,
                fragment_shader,
                geometry_shader,
                shader_program,
                position_attr,
                color_attr,
                norm_to_viewer_uniform,
                line_thickness_uniform,
            };

            shader.activate();

            Ok(shader)
        }
    }

    unsafe fn bind_attributes_to_vertex_array(&self) {
        gl::VertexAttribPointer(
            self.position_attr as gl::types::GLuint,
            POS_SIZE as GLint,
            gl::FLOAT,
            gl::FALSE,
            ((POS_SIZE + COLOR_SIZE) as usize * std::mem::size_of::<f32>()) as gl::types::GLsizei,
            std::ptr::null(),
        );

        gl::VertexAttribPointer(
            self.color_attr as gl::types::GLuint,
            COLOR_SIZE as GLint,
            gl::FLOAT,
            gl::FALSE,
            ((POS_SIZE + COLOR_SIZE) as usize * std::mem::size_of::<f32>()) as gl::types::GLsizei,
            (POS_SIZE as usize * std::mem::size_of::<f32>()) as *const c_void,
        );

        gl::EnableVertexAttribArray(self.position_attr as gl::types::GLuint);
        gl::EnableVertexAttribArray(self.color_attr as gl::types::GLuint);
    }

    unsafe fn activate(&self) {
        gl::UseProgram(self.shader_program);
    }
}

impl Drop for LineShader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.shader_program);
            gl::DeleteShader(self.fragment_shader);
            gl::DeleteShader(self.vertex_shader);
            gl::DeleteShader(self.geometry_shader);
        }
    }
}
