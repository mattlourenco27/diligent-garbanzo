use core::ffi::{c_void, CStr};

use gl::types::{GLchar, GLenum, GLint, GLsizei, GLuint};
use num_traits::ConstZero;
use sdl2::{
    pixels::Color,
    video::{GLContext, Window},
};

use crate::{
    matrix::Matrix3x3,
    objects::{svg::*, Object, ObjectMgr},
    render::{Renderer, Viewer},
    vector::{Vector2D, Vector3D},
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

enum VertexData {
    Combined(CombinedVertexData),
    Polygon(PolygonFillData),
}

struct CombinedVertexData {
    data: Vec<f32>,
    data_types: Vec<(GLenum, u32)>,
}

struct PolygonFillData {
    vertices: Vec<f32>,
    fill_sequence: Vec<GLuint>,
}

struct VertexExtractor {
    data: Vec<VertexData>,
}

impl VertexExtractor {
    fn from_svg_vertices(svg_object: &SVG) -> Self {
        let mut extractor = Self { data: Vec::new() };

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
            EmptyTag::Polygon(polygon) => self.load_polygon_vertices(polygon, transform),
            EmptyTag::Polyline(_polyline) => unimplemented!(),
            EmptyTag::Rect(_rect) => unimplemented!(),
        }
    }

    fn load_point(&mut self, point: &Point, transform: &Transform) {
        let new_transform = transform * &point.style.transform;
        let transformed_position = Vector3D::from_vector(&point.position) * new_transform;

        let color: GLColor = if point.style.fill_color == Style::DEFAULT.fill_color {
            point.style.stroke_color
        } else {
            point.style.fill_color
        }
        .into();

        if color.3 == 0.0 {
            return;
        }

        self.append_combined_data(
            gl::POINTS,
            &[
                transformed_position[0],
                transformed_position[1],
                color.0,
                color.1,
                color.2,
                color.3,
            ],
            1,
            true,
        );
    }

    fn load_line_vertices(&mut self, line: &Line, transform: &Transform) {
        let new_transform = transform * &line.style.transform;
        let transformed_p1 = Vector3D::from_vector(&line.from) * &new_transform;
        let transformed_p2 = Vector3D::from_vector(&line.to) * new_transform;

        let color: GLColor = if line.style.fill_color == Style::DEFAULT.fill_color {
            line.style.stroke_color
        } else {
            line.style.fill_color
        }
        .into();

        if color.3 == 0.0 {
            return;
        }

        self.append_combined_data(
            gl::LINES,
            &[
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
            true,
        );
    }

    fn append_combined_data(
        &mut self,
        data_type: GLenum,
        new_data: &[f32],
        num_vertices: u32,
        merge_with_last_if_possible: bool,
    ) {
        let combined_data = match self.data.last_mut() {
            Some(VertexData::Combined(combined_data)) => combined_data,
            _ => {
                self.data.push(VertexData::Combined(CombinedVertexData {
                    data: Vec::new(),
                    data_types: Vec::new(),
                }));
                match self.data.last_mut() {
                    Some(VertexData::Combined(combined_data)) => combined_data,
                    _ => panic!("Just pushed a CombinedVertexData, so this should not be None"),
                }
            }
        };

        combined_data.data.extend_from_slice(new_data);

        match combined_data.data_types.last_mut() {
            Some((last_data_type, last_count))
                if *last_data_type == data_type && merge_with_last_if_possible =>
            {
                *last_count += num_vertices;
            }
            _ => {
                combined_data.data_types.push((data_type, num_vertices));
            }
        }
    }

    fn load_polygon_vertices(&mut self, polygon: &Polygon, transform: &Transform) {
        let polygon_transform = transform * &polygon.style.transform;
        let mut fill_vertex_data: Vec<f32> = Vec::new();
        let mut fill_element_data: Vec<GLuint> = Vec::new();
        let mut stroke_vertex_data: Vec<f32> = Vec::new();
        let fill_color: GLColor = polygon.style.fill_color.into();
        let stroke_color: GLColor = polygon.style.stroke_color.into();
        let do_outline = stroke_color.3 > 0.0 && polygon.style.stroke_width > 0.0;
        let mut do_fill = fill_color.3 > 0.0;

        let triangles = if do_fill {
            crate::render::triangulation::triangulate(&polygon.points)
        } else {
            None
        };
        do_fill &= triangles.is_some();

        if !do_outline && !do_fill {
            return;
        }

        if do_fill {
            fill_vertex_data.reserve_exact(polygon.points.len() * (POS_SIZE + COLOR_SIZE) as usize);
            let triangles = triangles.unwrap();
            fill_element_data.reserve_exact(triangles.len() * 3);
            for triangle in triangles.iter() {
                fill_element_data.push(triangle[0] as GLuint);
                fill_element_data.push(triangle[1] as GLuint);
                fill_element_data.push(triangle[2] as GLuint);
            }
        }

        if do_outline {
            stroke_vertex_data
                .reserve_exact(polygon.points.len() * (POS_SIZE + COLOR_SIZE) as usize);
        }

        for point in polygon.points.iter() {
            let transformed_position = Vector3D::from_vector(point) * &polygon_transform;

            if do_fill {
                fill_vertex_data.extend_from_slice(&[
                    transformed_position[0],
                    transformed_position[1],
                    fill_color.0,
                    fill_color.1,
                    fill_color.2,
                    fill_color.3,
                ]);
            }

            if do_outline {
                stroke_vertex_data.extend_from_slice(&[
                    transformed_position[0],
                    transformed_position[1],
                    stroke_color.0,
                    stroke_color.1,
                    stroke_color.2,
                    stroke_color.3,
                ]);
            }
        }

        if do_fill {
            self.data.push(VertexData::Polygon(PolygonFillData {
                vertices: fill_vertex_data,
                fill_sequence: fill_element_data,
            }));
        }

        if do_outline {
            self.append_combined_data(
                gl::LINE_LOOP,
                &stroke_vertex_data,
                polygon.points.len() as u32,
                false,
            );
        }
    }
}

struct CombinedVertexArray {
    array_index: GLuint,
    buffer_index: GLuint,
    data_types: Vec<(GLenum, u32)>,
}

impl CombinedVertexArray {
    unsafe fn render(&self) {
        gl::BindVertexArray(self.array_index);
        gl::BindBuffer(gl::ARRAY_BUFFER, self.buffer_index);
        let mut total_drawn: u32 = 0;
        for (data_type, count) in self.data_types.iter() {
            gl::DrawArrays(*data_type, total_drawn as GLint, *count as GLsizei);
            total_drawn += *count;
        }
    }
}

impl Drop for CombinedVertexArray {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.buffer_index);
            gl::DeleteVertexArrays(1, &self.array_index);
        }
    }
}

struct ElementVertexArray {
    array_index: GLuint,
    buffer_index: GLuint,
    element_buffer_index: GLuint,
    num_elements: u32,
}

impl ElementVertexArray {
    unsafe fn render(&self) {
        gl::BindVertexArray(self.array_index);
        gl::BindBuffer(gl::ARRAY_BUFFER, self.buffer_index);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.element_buffer_index);
        gl::DrawElements(
            gl::TRIANGLES,
            self.num_elements as GLsizei,
            gl::UNSIGNED_INT,
            std::ptr::null(),
        );
    }
}

impl Drop for ElementVertexArray {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.element_buffer_index);
            gl::DeleteBuffers(1, &self.buffer_index);
            gl::DeleteVertexArrays(1, &self.array_index);
        }
    }
}

enum VertexArray {
    Combined(CombinedVertexArray),
    Element(ElementVertexArray),
}

impl VertexArray {
    fn gen_from_svg(svg_object: &SVG, shaders: &ShadersAndProgram) -> Vec<Self> {
        let vertices = VertexExtractor::from_svg_vertices(svg_object);

        let mut vertex_arrays = Vec::new();

        vertex_arrays.reserve_exact(vertices.data.len());

        for vertex_data in vertices.data.into_iter() {
            match vertex_data {
                VertexData::Combined(combined_data) => {
                    let mut combined_buffer = CombinedVertexArray {
                        array_index: 0,
                        buffer_index: 0,
                        data_types: combined_data.data_types,
                    };

                    unsafe {
                        gl::GenVertexArrays(1, &mut combined_buffer.array_index);
                        gl::BindVertexArray(combined_buffer.array_index);

                        gl::GenBuffers(1, &mut combined_buffer.buffer_index);
                        gl::BindBuffer(gl::ARRAY_BUFFER, combined_buffer.buffer_index);
                        gl::BufferData(
                            gl::ARRAY_BUFFER,
                            (combined_data.data.len() * std::mem::size_of::<f32>())
                                as gl::types::GLsizeiptr,
                            combined_data.data.as_ptr() as *const c_void,
                            gl::STATIC_DRAW,
                        );

                        shaders.bind_attributes_to_vertex_array();
                    }

                    vertex_arrays.push(VertexArray::Combined(combined_buffer));
                }
                VertexData::Polygon(polygon_data) => {
                    let mut element_buffer = ElementVertexArray {
                        array_index: 0,
                        buffer_index: 0,
                        element_buffer_index: 0,
                        num_elements: polygon_data.fill_sequence.len() as u32,
                    };

                    unsafe {
                        gl::GenVertexArrays(1, &mut element_buffer.array_index);
                        gl::BindVertexArray(element_buffer.array_index);

                        gl::GenBuffers(1, &mut element_buffer.buffer_index);
                        gl::BindBuffer(gl::ARRAY_BUFFER, element_buffer.buffer_index);
                        gl::BufferData(
                            gl::ARRAY_BUFFER,
                            (polygon_data.vertices.len() * std::mem::size_of::<f32>())
                                as gl::types::GLsizeiptr,
                            polygon_data.vertices.as_ptr() as *const c_void,
                            gl::STATIC_DRAW,
                        );

                        gl::GenBuffers(1, &mut element_buffer.element_buffer_index);
                        gl::BindBuffer(
                            gl::ELEMENT_ARRAY_BUFFER,
                            element_buffer.element_buffer_index,
                        );
                        gl::BufferData(
                            gl::ELEMENT_ARRAY_BUFFER,
                            (polygon_data.fill_sequence.len() * std::mem::size_of::<GLuint>())
                                as gl::types::GLsizeiptr,
                            polygon_data.fill_sequence.as_ptr() as *const c_void,
                            gl::STATIC_DRAW,
                        );

                        shaders.bind_attributes_to_vertex_array();
                    }

                    vertex_arrays.push(VertexArray::Element(element_buffer))
                }
            }
        }

        vertex_arrays
    }

    fn render(&self) {
        unsafe {
            match self {
                VertexArray::Combined(combined_buffer) => {
                    combined_buffer.render();
                }
                VertexArray::Element(element_buffer) => {
                    element_buffer.render();
                }
            }
        }
    }
}

struct GLViewer {
    center: Vector2D<f32>,
    zoom: f32,
    window_width_over_height: f32,
    norm_to_self_transform: Matrix3x3<f32>,
}

impl Viewer for GLViewer {
    fn center_on_object(&mut self, object: &Object) {
        let object_radius = object.svg_inst.dimension.clone() * 0.5;
        self.center[0] = object.position[0] + object_radius[0];
        self.center[1] = object.position[1] + object_radius[1];

        // In OpenGL, screen coordinates range from -1.0 to 1.0.
        // So screen width and height is always 2.0.
        let zoom_x = 2.0 / object.svg_inst.dimension[0];
        let zoom_y = 2.0 / object.svg_inst.dimension[1];

        self.zoom = std::cmp::min_by(zoom_x, zoom_y, |x, y| x.partial_cmp(y).unwrap());

        if self.zoom.is_infinite() {
            self.zoom = 1.0;
        }

        self.update_norm_to_self_transform();
    }

    fn move_to(&mut self, new_center: Vector2D<f32>) {
        self.center = new_center;
        self.update_norm_to_self_transform();
    }

    fn move_by(&mut self, delta_center: Vector2D<f32>) {
        self.center += delta_center * (1.0 / self.zoom);
        self.update_norm_to_self_transform();
    }

    fn zoom_to(&mut self, new_zoom: f32) {
        self.zoom = new_zoom;
        self.update_norm_to_self_transform();
    }

    fn zoom_by(&mut self, zoom_modifier: f32) {
        self.zoom *= zoom_modifier;
        self.update_norm_to_self_transform();
    }
}

impl GLViewer {
    fn new(window_size: Vector2D<u32>) -> Self {
        const DEFAULT_CENTER: Vector2D<f32> = Vector2D::ZERO;
        const DEFAULT_ZOOM: f32 = 1.0;

        let window_width_over_height = window_size[0] as f32 / window_size[1] as f32;
        Self {
            center: DEFAULT_CENTER,
            zoom: DEFAULT_ZOOM,
            norm_to_self_transform: Self::generate_norm_to_self_transform(
                &DEFAULT_CENTER,
                DEFAULT_ZOOM,
                window_width_over_height,
            ),
            window_width_over_height,
        }
    }

    fn norm_to_viewer(&self, position: &Vector2D<f32>) -> Vector2D<f32> {
        let transformed = Vector3D::from_vector(position) * &self.norm_to_self_transform;
        Vector2D::from_vector(&transformed)
    }

    fn get_norm_to_viewer(&self) -> &Matrix3x3<f32> {
        &self.norm_to_self_transform
    }

    fn generate_norm_to_self_transform(
        center: &Vector2D<f32>,
        zoom: f32,
        width_over_height: f32,
    ) -> Matrix3x3<f32> {
        // Translate to viewer position
        let mut position_matrix = Matrix3x3::IDENTITY3X3;
        position_matrix[2][0] = -center[0];
        position_matrix[2][1] = -center[1];

        // Zoom the appropriate amount
        let mut zoom_matrix = Matrix3x3::IDENTITY3X3;

        if width_over_height > 1.0 {
            zoom_matrix[0][0] = zoom / width_over_height;
            zoom_matrix[1][1] = zoom;
        } else {
            zoom_matrix[0][0] = zoom;
            zoom_matrix[1][1] = zoom * width_over_height;
        }

        &position_matrix * &zoom_matrix
    }

    fn update_norm_to_self_transform(&mut self) {
        self.norm_to_self_transform = Self::generate_norm_to_self_transform(
            &self.center,
            self.zoom,
            self.window_width_over_height,
        );
    }
}

pub struct GLRenderer {
    window: Window,
    _gl_ctx: GLContext,
    viewer: GLViewer,
    shaders: ShadersAndProgram,
    vertex_arrays: Vec<VertexArray>,
}

impl GLRenderer {
    pub fn new(window: Window, gl_ctx: GLContext, object_mgr: &ObjectMgr) -> Result<Self, String> {
        let window_size: [u32; 2] = window.size().into();

        let shaders = ShadersAndProgram::build()?;

        let mut vertex_arrays = Vec::new();
        for object in object_mgr.get_objects() {
            vertex_arrays.extend(VertexArray::gen_from_svg(&object.svg_inst, &shaders));
        }

        let gl_renderer = Self {
            window,
            _gl_ctx: gl_ctx,
            viewer: GLViewer::new(Vector2D::from(window_size)),
            shaders,
            vertex_arrays,
        };

        Ok(gl_renderer)
    }

    fn render_object(&self, vertex_array: &VertexArray) {
        vertex_array.render();
    }
}

impl Renderer for GLRenderer {
    fn get_viewer(&mut self) -> &mut dyn Viewer {
        &mut self.viewer
    }

    fn clear(&mut self) {
        unsafe {
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    fn render_objects(&mut self) {
        // update uniform controlling the viewer transform (if necessary? Maybe do that only when it updates?)
        self.shaders
            .update_uniform(self.viewer.get_norm_to_viewer());

        for vertex_array in self.vertex_arrays.iter() {
            self.render_object(vertex_array);
        }
    }

    fn present(&mut self) {
        self.window.gl_swap_window();
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        objects::{svg::SVG, Object},
        render::Viewer,
        vector::{Vector2D, Vector3D},
    };

    use super::GLViewer;

    fn new_viewer() -> GLViewer {
        GLViewer::new(Vector2D::from([100, 100]))
    }

    #[test]
    fn init_at_origin() {
        let viewer = new_viewer();
        assert_eq!(viewer.center, Vector2D::from([0.0, 0.0]));
    }

    #[test]
    fn pixels_at_viewer_center_map_to_the_screen_center() {
        let viewer = new_viewer();
        assert_eq!(
            viewer.norm_to_viewer(&viewer.center),
            Vector2D::from([0.0, 0.0])
        );
    }

    #[test]
    fn pixels_at_screen_center_are_unaffected_by_zoom() {
        let mut viewer = new_viewer();
        let pixel_mapping_before_zoom = viewer.norm_to_viewer(&viewer.center);
        viewer.zoom_by(2.0);
        let pixel_mapping_after_zoom = viewer.norm_to_viewer(&viewer.center);
        assert_eq!(pixel_mapping_before_zoom, pixel_mapping_after_zoom);
    }

    #[test]
    fn zoom_value_of_1_does_not_change_position_norm() {
        const ZOOM_AMOUNT: f32 = 1.0;

        let mut viewer = new_viewer();
        let screen_center = viewer.norm_to_viewer(&viewer.center);
        viewer.zoom_to(ZOOM_AMOUNT);

        let pixel = Vector2D::from([3.0, 4.0]);
        let position_norm_before_mapping = pixel.get_norm();
        let position_norm_after_mapping =
            (viewer.norm_to_viewer(&pixel) - screen_center).get_norm();

        assert_eq!(position_norm_before_mapping, position_norm_after_mapping);
    }

    #[test]
    fn zooming_moves_pixels_away_from_the_screen_center_by_the_same_amount() {
        const ZOOM_AMOUNT: f32 = 3.77;

        let mut viewer = new_viewer();
        let screen_center = viewer.norm_to_viewer(&viewer.center);

        let pixel = Vector2D::from([3.0, 4.0]);
        let position_norm_before_zoom = (viewer.norm_to_viewer(&pixel) - &screen_center).get_norm();

        viewer.zoom_by(ZOOM_AMOUNT);

        let position_norm_after_zoom = (viewer.norm_to_viewer(&pixel) - &screen_center).get_norm();

        assert_eq!(
            position_norm_before_zoom * ZOOM_AMOUNT,
            position_norm_after_zoom
        );
    }

    #[test]
    fn viewer_centers_on_a_given_object() {
        let mut viewer = new_viewer();
        let object = Object {
            position: Vector3D::from([4.0, -3.0, 1.0]),
            svg_inst: SVG {
                dimension: Vector2D::from([20.0, 20.0]),
                elements: Vec::new(),
            },
        };

        viewer.center_on_object(&object);

        assert_eq!(
            viewer.center,
            Vector2D::from([(20.0 / 2.0) + 4.0, (20.0 / 2.0) - 3.0])
        );
    }

    #[test]
    fn viewer_zooms_to_largest_dimension_of_object() {
        let mut viewer = new_viewer();
        let object = Object {
            position: Vector3D::from([4.0, -3.0, 1.0]),
            svg_inst: SVG {
                dimension: Vector2D::from([10.0, 25.0]),
                elements: Vec::new(),
            },
        };

        viewer.center_on_object(&object);

        assert_eq!(viewer.zoom, 2.0 / 25.0)
    }

    #[test]
    fn viewer_shouldnt_zoom_infinitely_when_object_size_is_zero() {
        let mut viewer = new_viewer();
        let object = Object {
            position: Vector3D::from([4.0, -3.0, 1.0]),
            svg_inst: SVG {
                dimension: Vector2D::from([0.0, 0.0]),
                elements: Vec::new(),
            },
        };

        viewer.center_on_object(&object);

        assert_ne!(viewer.zoom, f32::INFINITY)
    }

    #[test]
    fn viewer_centers_itself_on_position_to_move_to() {
        let mut viewer = new_viewer();
        let new_center = Vector2D::from([5.0, -5.0]);

        viewer.move_to(new_center.clone());

        assert_eq!(viewer.center, new_center);
    }

    #[test]
    fn viewer_moves_by_amount_specified_divided_by_zoom() {
        const ZOOM_AMOUNT: f32 = 5.0;
        let delta_position = Vector2D::from([5.0, -5.0]);

        let mut viewer = new_viewer();
        viewer.zoom_to(ZOOM_AMOUNT);

        viewer.move_by(delta_position.clone());
        let center_after_move = viewer.center.clone();

        assert_eq!(delta_position * (1.0 / ZOOM_AMOUNT), center_after_move);
    }
}
