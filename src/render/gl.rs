use core::ffi::{c_void, CStr};

use gl::types::{GLchar, GLenum, GLint, GLsizei, GLuint};
use sdl2::{
    pixels::Color,
    video::{GLContext, Window},
};

use crate::{
    matrix::Matrix3x3,
    objects::{
        svg::{Element, EmptyTag, Line, Point, StartTag, Style, Transform, SVG},
        ObjectMgr,
    },
    vector::{Vector2D, Vector3D},
    viewer::Viewer,
};

const POS_SIZE: u8 = 2;
const COLOR_SIZE: u8 = 4;

const VERTEX_SHADER: &CStr = c"#version 150 core

in vec2 position;
in vec4 color;

uniform mat3 norm_to_viewer;

out vec4 Color;

void main()
{
    Color = color;
    vec3 transformed_position = vec3(position, 1.0) * norm_to_viewer;
    gl_Position = vec4(transformed_position.x, -transformed_position.y, 0.0, 1.0);
}";

const FRAGMENT_SHADER: &CStr = c"#version 150 core

in vec4 Color;

out vec4 outColor;

void main()
{
    outColor = Color;
}";

unsafe fn maybe_get_gl_error() -> Result<(), String> {
    let error = gl::GetError();
    if error != gl::NO_ERROR {
        return Err(format!("Error code: {error}"));
    }

    Ok(())
}

struct ShadersAndProgram {
    vertex_shader: GLuint,
    fragment_shader: GLuint,
    shader_program: GLuint,
    position_attr: GLuint,
    color_attr: GLuint,
    norm_to_viewer_uniform: GLuint,
}

impl ShadersAndProgram {
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

        ShadersAndProgram::maybe_get_shader_compile_error(shader_id)?;

        Ok(shader_id)
    }

    unsafe fn create_new_program(
        vertex_shader: GLuint,
        fragment_shader: GLuint,
    ) -> Result<GLuint, String> {
        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);

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

    unsafe fn activate_program(shader_program: GLuint) -> Result<(), String> {
        gl::UseProgram(shader_program);

        maybe_get_gl_error()?;

        Ok(())
    }

    unsafe fn get_attributes(shader_program: GLuint) -> Result<(GLuint, GLuint, GLuint), String> {
        let position_attribute = gl::GetAttribLocation(shader_program, c"position".as_ptr());

        maybe_get_gl_error()?;

        let color_attribute = gl::GetAttribLocation(shader_program, c"color".as_ptr());

        maybe_get_gl_error()?;

        let norm_to_viewer_uniform =
            gl::GetUniformLocation(shader_program, c"norm_to_viewer".as_ptr());

        maybe_get_gl_error()?;

        Ok((
            position_attribute as GLuint,
            color_attribute as GLuint,
            norm_to_viewer_uniform as GLuint,
        ))
    }

    fn build() -> Result<ShadersAndProgram, String> {
        unsafe {
            let vertex_shader =
                ShadersAndProgram::send_and_compile_shader(gl::VERTEX_SHADER, VERTEX_SHADER)?;
            let fragment_shader =
                ShadersAndProgram::send_and_compile_shader(gl::FRAGMENT_SHADER, FRAGMENT_SHADER)?;

            let shader_program =
                ShadersAndProgram::create_new_program(vertex_shader, fragment_shader)?;

            ShadersAndProgram::bind_fragment_shader_output(shader_program)?;

            ShadersAndProgram::link_program(shader_program)?;

            ShadersAndProgram::activate_program(shader_program)?;

            let (position_attr, color_attr, norm_to_viewer_uniform) =
                ShadersAndProgram::get_attributes(shader_program)?;

            Ok(ShadersAndProgram {
                vertex_shader,
                fragment_shader,
                shader_program,
                position_attr,
                color_attr,
                norm_to_viewer_uniform,
            })
        }
    }

    fn bind_attributes_to_vertex_array(&self) -> Result<(), String> {
        unsafe {
            gl::VertexAttribPointer(
                self.position_attr as gl::types::GLuint,
                POS_SIZE as GLint,
                gl::FLOAT,
                gl::FALSE,
                ((POS_SIZE + COLOR_SIZE) as usize * std::mem::size_of::<f32>())
                    as gl::types::GLsizei,
                std::ptr::null(),
            );

            maybe_get_gl_error()?;

            gl::VertexAttribPointer(
                self.color_attr as gl::types::GLuint,
                COLOR_SIZE as GLint,
                gl::FLOAT,
                gl::FALSE,
                ((POS_SIZE + COLOR_SIZE) as usize * std::mem::size_of::<f32>())
                    as gl::types::GLsizei,
                (POS_SIZE as usize * std::mem::size_of::<f32>()) as *const c_void,
            );

            maybe_get_gl_error()?;

            gl::EnableVertexAttribArray(self.position_attr as gl::types::GLuint);
            gl::EnableVertexAttribArray(self.color_attr as gl::types::GLuint);
        }

        Ok(())
    }

    fn update_uniform(&self, norm_to_viewer_transform: &Matrix3x3<f32>) {
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
}

impl Drop for ShadersAndProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.shader_program);
            gl::DeleteShader(self.fragment_shader);
            gl::DeleteShader(self.vertex_shader);
        }
    }
}

struct GLColor(f32, f32, f32, f32);

impl From<Color> for GLColor {
    fn from(value: Color) -> Self {
        const U8_TO_F32: f32 = 1.0 / core::u8::MAX as f32;
        Self(
            value.r as f32 * U8_TO_F32,
            value.g as f32 * U8_TO_F32,
            value.b as f32 * U8_TO_F32,
            value.a as f32 * U8_TO_F32,
        )
    }
}

struct VertexExtractor {
    data: Vec<f32>,
    data_types: Vec<(GLenum, u32)>,
}

impl VertexExtractor {
    fn from_svg_vertices(svg_object: &SVG) -> Self {
        let mut extractor = Self {
            data: Vec::new(),
            data_types: Vec::new(),
        };

        for element in svg_object.elements.iter() {
            extractor.load_element_vertices(&element, &Matrix3x3::IDENTITY3X3);
        }

        extractor
    }

    fn load_svg_vertices(&mut self, svg_object: &SVG, transform: &Transform) {
        for element in svg_object.elements.iter() {
            self.load_element_vertices(&element, transform);
        }
    }

    fn load_element_vertices(&mut self, element: &Element, transform: &Transform) {
        match element {
            Element::StartTag(start_tag) => self.load_tag_group_vertices(start_tag, transform),
            Element::EmptyTag(empty_tag) => self.load_empty_tag_vertices(empty_tag, transform),
            Element::EndTag(_) => (),
        }
    }

    fn load_tag_group_vertices(&mut self, tag_group: &StartTag, transform: &Transform) {
        match tag_group {
            StartTag::Group(group) => {
                let new_transform = transform * &group.style.transform;
                for element in group.elements.iter() {
                    self.load_element_vertices(element, &new_transform);
                }
            }
            StartTag::SVG(svg_object) => self.load_svg_vertices(svg_object, transform),
        }
    }

    fn load_empty_tag_vertices(&mut self, empty_tag: &EmptyTag, transform: &Transform) {
        match empty_tag {
            EmptyTag::Ellipse(_ellipse) => unimplemented!(),
            EmptyTag::Image(_image) => unimplemented!(),
            EmptyTag::Line(line) => self.load_line_vertices(line, transform),
            EmptyTag::Point(point) => self.load_point(point, transform),
            EmptyTag::Polygon(_polygon) => unimplemented!(),
            EmptyTag::Polyline(_polyline) => unimplemented!(),
            EmptyTag::Rect(_rect) => unimplemented!(),
        }
    }

    fn load_point(&mut self, point: &Point, transform: &Transform) {
        let new_transform = transform * &point.style.transform;
        let transformed_position = Vector3D::from_vector(&point.position) * new_transform;

        let color: GLColor = (|| {
            if point.style.fill_color == Style::DEFAULT.fill_color {
                point.style.stroke_color
            } else {
                point.style.fill_color
            }
        })()
        .into();

        self.append_data(
            gl::POINTS,
            vec![
                transformed_position[0],
                transformed_position[1],
                color.0,
                color.1,
                color.2,
                color.3,
            ],
            1,
        );
    }

    fn load_line_vertices(&mut self, line: &Line, transform: &Transform) {
        let new_transform = transform * &line.style.transform;
        let transformed_p1 = Vector3D::from_vector(&line.from) * &new_transform;
        let transformed_p2 = Vector3D::from_vector(&line.to) * new_transform;

        let color: GLColor = (|| {
            if line.style.fill_color == Style::DEFAULT.fill_color {
                line.style.stroke_color
            } else {
                line.style.fill_color
            }
        })()
        .into();

        self.append_data(
            gl::LINES,
            vec![
                transformed_p1[0],
                transformed_p1[1],
                color.0,
                color.1,
                color.2,
                color.3,
                transformed_p2[0],
                transformed_p2[1],
                color.0,
                color.1,
                color.2,
                color.3,
            ],
            2,
        );
    }

    fn append_data(&mut self, data_type: GLenum, mut new_data: Vec<f32>, num_vertices: u32) {
        self.data.append(&mut new_data);

        match self.data_types.last_mut() {
            None => self.data_types.push((data_type, num_vertices)),
            Some(data_type_and_count) => {
                if data_type_and_count.0 == data_type {
                    data_type_and_count.1 += num_vertices;
                } else {
                    self.data_types.push((data_type, num_vertices));
                }
            }
        }
    }
}

struct VertexArray {
    array_index: GLuint,
    buffer_index: GLuint,
    data_types: Vec<(GLenum, u32)>,
}

impl VertexArray {
    fn from_svg(svg_object: &SVG) -> Self {
        let vertices = VertexExtractor::from_svg_vertices(svg_object);

        let mut vertex_array = Self {
            array_index: 0,
            buffer_index: 0,
            data_types: vertices.data_types,
        };

        unsafe {
            gl::GenVertexArrays(1, &mut vertex_array.array_index);
            gl::BindVertexArray(vertex_array.array_index);

            gl::GenBuffers(1, &mut vertex_array.buffer_index);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_array.buffer_index);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.data.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                vertices.data.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );
        }

        vertex_array
    }

    fn render(&self) {
        unsafe {
            gl::BindVertexArray(self.array_index);

            let mut total_drawn: u32 = 0;
            for (data_type, count) in self.data_types.iter() {
                gl::DrawArrays(*data_type, total_drawn as GLint, *count as GLsizei);
                total_drawn += *count;
            }
        }
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &mut self.buffer_index);
            gl::DeleteVertexArrays(1, &mut self.array_index);
        }
    }
}

pub struct Renderer {
    window: Window,
    _gl_ctx: GLContext,
    pub viewer: Viewer,
    shaders: ShadersAndProgram,
    vertex_arrays: Vec<VertexArray>,
}

impl Renderer {
    pub fn new(window: Window, gl_ctx: GLContext, object_mgr: &ObjectMgr) -> Result<Self, String> {
        let window_size: [u32; 2] = window.size().into();

        let shaders = ShadersAndProgram::build()?;

        let mut vertex_arrays = Vec::new();
        for object in object_mgr.get_objects() {
            vertex_arrays.push(VertexArray::from_svg(&object.svg_inst));
            shaders.bind_attributes_to_vertex_array()?;
        }

        let gl_renderer = Self {
            window,
            _gl_ctx: gl_ctx,
            viewer: Viewer::new(Vector2D::from(window_size)),
            shaders,
            vertex_arrays,
        };

        Ok(gl_renderer)
    }

    pub fn clear(&mut self) {
        unsafe {
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    pub fn render_objects(&self) {
        // update uniform controlling the viewer transform (if necessary? Maybe do that only when it updates?)
        self.shaders
            .update_uniform(self.viewer.get_norm_to_viewer());

        for vertex_array in self.vertex_arrays.iter() {
            self.render_object(vertex_array);
        }
    }

    pub fn present(&mut self) {
        self.window.gl_swap_window();
    }

    fn render_object(&self, vertex_array: &VertexArray) {
        vertex_array.render();
    }
}
