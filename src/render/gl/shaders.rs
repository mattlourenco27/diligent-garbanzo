use core::ffi::{c_void, CStr};
use gl::types::{GLchar, GLenum, GLint, GLuint};

use crate::matrix::Matrix3x3;

pub const POS_SIZE: u8 = 2;
pub const COLOR_SIZE: u8 = 4;

pub enum Shader {
    Basic,
    Line,
    LineAdjacency,
}

pub struct ShaderMgr {
    basic_shader: BasicShader,
    line_shader: LineShader,
    line_adjacency_shader: LineAdjacencyShader,
    active_shader: Shader,
}

impl ShaderMgr {
    pub fn new() -> Result<Self, String> {
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        Ok(Self {
            basic_shader: BasicShader::build()?,
            line_shader: LineShader::build()?,
            line_adjacency_shader: LineAdjacencyShader::build()?,
            active_shader: Shader::Basic,
        })
    }

    pub unsafe fn activate(&mut self, shader: Shader) {
        match shader {
            Shader::Basic => self.basic_shader.activate(),
            Shader::Line => self.line_shader.activate(),
            Shader::LineAdjacency => self.line_adjacency_shader.activate(),
        }
        self.active_shader = shader;
    }

    pub unsafe fn bind_attributes_to_vertex_array(&self) {
        match self.active_shader {
            Shader::Basic => self.basic_shader.attributes.bind(),
            Shader::Line => self.line_shader.attributes.bind(),
            Shader::LineAdjacency => self.line_adjacency_shader.attributes.bind(),
        }
    }

    pub unsafe fn update_norm_to_viewer(&mut self, norm_to_viewer_transform: &Matrix3x3<f32>) {
        self.basic_shader.activate();
        self.basic_shader
            .attributes
            .norm_to_viewer
            .update(norm_to_viewer_transform.clone());
        self.line_shader.activate();
        self.line_shader
            .attributes
            .norm_to_viewer
            .update(norm_to_viewer_transform.clone());
        self.line_adjacency_shader.activate();
        self.line_adjacency_shader
            .attributes
            .norm_to_viewer
            .update(norm_to_viewer_transform.clone());

        match self.active_shader {
            Shader::Basic => self.basic_shader.activate(),
            Shader::Line => self.line_shader.activate(),
            Shader::LineAdjacency => self.line_adjacency_shader.activate(),
        }
    }

    pub unsafe fn set_svg_transform(&mut self, svg_transform: Matrix3x3<f32>) {
        match self.active_shader {
            Shader::Basic => self
                .basic_shader
                .attributes
                .svg_transform
                .update(svg_transform),
            Shader::Line => self
                .line_shader
                .attributes
                .svg_transform
                .update(svg_transform),
            Shader::LineAdjacency => self
                .line_adjacency_shader
                .attributes
                .svg_transform
                .update(svg_transform),
        }
    }

    pub unsafe fn set_line_thickness(&mut self, thickness: f32) {
        match self.active_shader {
            Shader::Line => self.line_shader.attributes.thickness.update(thickness),
            Shader::LineAdjacency => self
                .line_adjacency_shader
                .attributes
                .thickness
                .update(thickness),
            _ => panic!("Tried to update line thickness on a shader that does not support it."),
        }
    }
}

trait ShaderProgram {
    fn build() -> Result<Self, String>
    where
        Self: Sized;

    unsafe fn activate(&self);
}

struct Uniform<T>
where
    T: PartialEq,
{
    uniform_index: GLint,
    current_value: Option<T>,
}

impl Uniform<f32> {
    fn update(&mut self, new_value: f32) {
        match &self.current_value {
            Some(value) if *value == new_value => return,
            _ => {}
        }

        unsafe {
            gl::Uniform1f(self.uniform_index, new_value);
        }

        self.current_value = Some(new_value);
    }
}

impl Uniform<Matrix3x3<f32>> {
    fn update(&mut self, new_value: Matrix3x3<f32>) {
        match &self.current_value {
            Some(value) if *value == new_value => return,
            _ => {}
        }

        unsafe {
            gl::UniformMatrix3fv(
                self.uniform_index,
                1,
                gl::TRUE,
                <&Matrix3x3<f32> as Into<&[[f32; 3]; 3]>>::into(&new_value)[0].as_ptr(),
            );
        }

        self.current_value = Some(new_value);
    }
}

trait Attributes {
    fn get_position_index(&self) -> GLuint;
    fn get_color_index(&self) -> GLuint;

    unsafe fn bind(&self) {
        gl::VertexAttribPointer(
            self.get_position_index() as gl::types::GLuint,
            POS_SIZE as GLint,
            gl::FLOAT,
            gl::FALSE,
            ((POS_SIZE + COLOR_SIZE) as usize * std::mem::size_of::<f32>()) as gl::types::GLsizei,
            std::ptr::null(),
        );

        gl::VertexAttribPointer(
            self.get_color_index() as gl::types::GLuint,
            COLOR_SIZE as GLint,
            gl::FLOAT,
            gl::FALSE,
            ((POS_SIZE + COLOR_SIZE) as usize * std::mem::size_of::<f32>()) as gl::types::GLsizei,
            (POS_SIZE as usize * std::mem::size_of::<f32>()) as *const c_void,
        );

        gl::EnableVertexAttribArray(self.get_position_index() as gl::types::GLuint);
        gl::EnableVertexAttribArray(self.get_color_index() as gl::types::GLuint);
    }
}

struct BasicAttributes {
    position: GLuint,
    color: GLuint,
    norm_to_viewer: Uniform<Matrix3x3<f32>>,
    svg_transform: Uniform<Matrix3x3<f32>>,
}

impl Attributes for BasicAttributes {
    fn get_position_index(&self) -> GLuint {
        self.position
    }

    fn get_color_index(&self) -> GLuint {
        self.color
    }
}

impl BasicAttributes {
    unsafe fn new(shader_program: GLuint) -> Result<Self, String> {
        let position = gl::GetAttribLocation(shader_program, c"position".as_ptr());
        maybe_get_gl_error()?;

        let color = gl::GetAttribLocation(shader_program, c"color".as_ptr());
        maybe_get_gl_error()?;

        let norm_to_viewer = gl::GetUniformLocation(shader_program, c"norm_to_viewer".as_ptr());
        maybe_get_gl_error()?;

        let svg_transform = gl::GetUniformLocation(shader_program, c"svg_transform".as_ptr());
        maybe_get_gl_error()?;

        Ok(BasicAttributes {
            position: position as GLuint,
            color: color as GLuint,
            norm_to_viewer: Uniform {
                uniform_index: norm_to_viewer,
                current_value: None,
            },
            svg_transform: Uniform {
                uniform_index: svg_transform,
                current_value: None,
            },
        })
    }
}

struct LineAttributes {
    position: GLuint,
    color: GLuint,
    norm_to_viewer: Uniform<Matrix3x3<f32>>,
    svg_transform: Uniform<Matrix3x3<f32>>,
    thickness: Uniform<f32>,
}

impl Attributes for LineAttributes {
    fn get_position_index(&self) -> GLuint {
        self.position
    }

    fn get_color_index(&self) -> GLuint {
        self.color
    }
}

impl LineAttributes {
    unsafe fn new(shader_program: GLuint) -> Result<Self, String> {
        let position = gl::GetAttribLocation(shader_program, c"position".as_ptr());
        maybe_get_gl_error()?;

        let color = gl::GetAttribLocation(shader_program, c"color".as_ptr());
        maybe_get_gl_error()?;

        let norm_to_viewer = gl::GetUniformLocation(shader_program, c"norm_to_viewer".as_ptr());
        maybe_get_gl_error()?;

        let svg_transform = gl::GetUniformLocation(shader_program, c"svg_transform".as_ptr());
        maybe_get_gl_error()?;

        let thickness = gl::GetUniformLocation(shader_program, c"thickness".as_ptr());
        maybe_get_gl_error()?;

        Ok(LineAttributes {
            position: position as GLuint,
            color: color as GLuint,
            norm_to_viewer: Uniform {
                uniform_index: norm_to_viewer,
                current_value: None,
            },
            svg_transform: Uniform {
                uniform_index: svg_transform,
                current_value: None,
            },
            thickness: Uniform {
                uniform_index: thickness,
                current_value: None,
            },
        })
    }
}

struct LineShader {
    vertex_shader: GLuint,
    fragment_shader: GLuint,
    geometry_shader: GLuint,
    shader_program: GLuint,
    attributes: LineAttributes,
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
uniform mat3 svg_transform;

void EmitTransformedVertex(in vec2 position) {
    vec3 transformed = vec3(position, 1.0) * svg_transform * norm_to_viewer;
    gl_Position = vec4(transformed.x, -transformed.y, 0.0, 1.0);
    EmitVertex();
}

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

    GeoColor = VertexColor[0];
    EmitTransformedVertex(v0);
    EmitTransformedVertex(v1);
    
    GeoColor = VertexColor[1];
    EmitTransformedVertex(v2);
    EmitTransformedVertex(v3);
    
    EndPrimitive();
}";
}

impl LineShader {
    unsafe fn bind_fragment_shader_output(&self) -> Result<(), String> {
        gl::BindFragDataLocation(self.shader_program, 0, c"outColor".as_ptr());

        maybe_get_gl_error()?;

        Ok(())
    }
}

impl ShaderProgram for LineShader {
    fn build() -> Result<LineShader, String> {
        unsafe {
            let shader_program = create_program()?;
            let vertex_shader = send_compile_and_attach_shader(
                gl::VERTEX_SHADER,
                LineShader::VERTEX_SHADER,
                shader_program,
            )?;
            let fragment_shader = send_compile_and_attach_shader(
                gl::FRAGMENT_SHADER,
                LineShader::FRAGMENT_SHADER,
                shader_program,
            )?;
            let geometry_shader = send_compile_and_attach_shader(
                gl::GEOMETRY_SHADER,
                LineShader::GEOMETRY_SHADER,
                shader_program,
            )?;

            link_program(shader_program)?;

            let shader = LineShader {
                vertex_shader,
                fragment_shader,
                geometry_shader,
                shader_program,
                attributes: LineAttributes::new(shader_program)?,
            };

            shader.bind_fragment_shader_output()?;

            Ok(shader)
        }
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

struct LineAdjacencyShader {
    vertex_shader: GLuint,
    fragment_shader: GLuint,
    geometry_shader: GLuint,
    shader_program: GLuint,
    attributes: LineAttributes,
}

impl LineAdjacencyShader {
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
layout(lines_adjacency) in;
layout(triangle_strip, max_vertices = 8) out;

in vec4 VertexColor[];
out vec4 GeoColor;

uniform float thickness;
uniform mat3 norm_to_viewer;
uniform mat3 svg_transform;

void EmitTransformedVertex(in vec2 position) {
    vec3 transformed = vec3(position, 1.0) * svg_transform * norm_to_viewer;
    gl_Position = vec4(transformed.x, -transformed.y, 0.0, 1.0);
    EmitVertex();
}

void main() {
    vec2 p0 = gl_in[0].gl_Position.xy; // previous point
    vec2 p1 = gl_in[1].gl_Position.xy; // current start
    vec2 p2 = gl_in[2].gl_Position.xy; // current end
    vec2 p3 = gl_in[3].gl_Position.xy; // next point

    vec2 v1 = normalize(p2 - p1); // current segment direction
    vec2 v2 = normalize(p3 - p2); // next segment direction

    vec2 n1 = vec2(-v1.y, v1.x) * thickness * 0.5;

    // Create basic line segment
    GeoColor = VertexColor[1];
    EmitTransformedVertex(p1 + n1);
    EmitTransformedVertex(p1 - n1);

    GeoColor = VertexColor[2];
    EmitTransformedVertex(p2 + n1);
    EmitTransformedVertex(p2 - n1);

    // Handle join at p2
    if (p2 != p3) {
        vec2 n2 = vec2(-v2.y, v2.x) * thickness * 0.5;
        
        EmitTransformedVertex(p2 + n2);
        EmitTransformedVertex(p2 - n2);
    }
    
    EndPrimitive();
}";
}

impl LineAdjacencyShader {
    unsafe fn bind_fragment_shader_output(&self) -> Result<(), String> {
        gl::BindFragDataLocation(self.shader_program, 0, c"outColor".as_ptr());

        maybe_get_gl_error()?;

        Ok(())
    }
}

impl ShaderProgram for LineAdjacencyShader {
    fn build() -> Result<LineAdjacencyShader, String> {
        unsafe {
            let shader_program = create_program()?;
            let vertex_shader = send_compile_and_attach_shader(
                gl::VERTEX_SHADER,
                LineAdjacencyShader::VERTEX_SHADER,
                shader_program,
            )?;
            let fragment_shader = send_compile_and_attach_shader(
                gl::FRAGMENT_SHADER,
                LineAdjacencyShader::FRAGMENT_SHADER,
                shader_program,
            )?;
            let geometry_shader = send_compile_and_attach_shader(
                gl::GEOMETRY_SHADER,
                LineAdjacencyShader::GEOMETRY_SHADER,
                shader_program,
            )?;

            link_program(shader_program)?;

            let shader = LineAdjacencyShader {
                vertex_shader,
                fragment_shader,
                geometry_shader,
                shader_program,
                attributes: LineAttributes::new(shader_program)?,
            };

            shader.bind_fragment_shader_output()?;

            Ok(shader)
        }
    }

    unsafe fn activate(&self) {
        gl::UseProgram(self.shader_program);
    }
}

impl Drop for LineAdjacencyShader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.shader_program);
            gl::DeleteShader(self.fragment_shader);
            gl::DeleteShader(self.vertex_shader);
            gl::DeleteShader(self.geometry_shader);
        }
    }
}

struct BasicShader {
    vertex_shader: GLuint,
    fragment_shader: GLuint,
    shader_program: GLuint,
    attributes: BasicAttributes,
}

impl BasicShader {
    const VERTEX_SHADER: &CStr = c"#version 150 core

in vec2 position;
in vec4 color;

uniform mat3 norm_to_viewer;
uniform mat3 svg_transform;

out vec4 Color;

void main() {
    Color = color;
    vec3 transformed_position = vec3(position, 1.0) * svg_transform * norm_to_viewer;
    gl_Position = vec4(transformed_position.x, -transformed_position.y, 0.0, 1.0);
}";

    const FRAGMENT_SHADER: &CStr = c"#version 150 core

in vec4 Color;

out vec4 outColor;

void main()
{
    outColor = Color;
}";
}

impl BasicShader {
    unsafe fn bind_fragment_shader_output(&self) -> Result<(), String> {
        gl::BindFragDataLocation(self.shader_program, 0, c"outColor".as_ptr());

        maybe_get_gl_error()?;

        Ok(())
    }
}

impl ShaderProgram for BasicShader {
    fn build() -> Result<BasicShader, String> {
        unsafe {
            let shader_program = create_program()?;
            let vertex_shader = send_compile_and_attach_shader(
                gl::VERTEX_SHADER,
                BasicShader::VERTEX_SHADER,
                shader_program,
            )?;
            let fragment_shader = send_compile_and_attach_shader(
                gl::FRAGMENT_SHADER,
                BasicShader::FRAGMENT_SHADER,
                shader_program,
            )?;

            link_program(shader_program)?;

            let shader = BasicShader {
                vertex_shader,
                fragment_shader,
                shader_program,
                attributes: BasicAttributes::new(shader_program)?,
            };

            shader.bind_fragment_shader_output()?;

            Ok(shader)
        }
    }

    unsafe fn activate(&self) {
        gl::UseProgram(self.shader_program);
    }
}

impl Drop for BasicShader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.shader_program);
            gl::DeleteShader(self.fragment_shader);
            gl::DeleteShader(self.vertex_shader);
        }
    }
}

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

unsafe fn send_compile_and_attach_shader(
    shader_type: GLenum,
    shader_source: &CStr,
    shader_program: GLuint,
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

    gl::AttachShader(shader_program, shader_id);

    maybe_get_gl_error()?;

    Ok(shader_id)
}

unsafe fn create_program() -> Result<GLuint, String> {
    let shader_program = gl::CreateProgram();

    maybe_get_gl_error()?;

    Ok(shader_program)
}

unsafe fn link_program(shader_program: GLuint) -> Result<(), String> {
    gl::LinkProgram(shader_program);

    maybe_get_gl_error()?;

    Ok(())
}
