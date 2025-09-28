use core::ffi::c_void;

use gl::types::{GLenum, GLint, GLsizei, GLuint};
use num_traits::ConstZero;
use sdl2::{
    pixels::Color,
    video::{GLContext, Window},
};

use crate::{
    matrix::Matrix3x3,
    objects::{svg::*, Object, ObjectMgr},
    render::{gl::shaders::{LineShader, ShaderProgram}, Renderer, Viewer},
    vector::{Vector2D, Vector3D},
};

mod shaders;

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

enum RawOperationData {
    DrawVertices(CombinedVertexData),
    FillPolygon(PolygonFillData),
    UpdateLineThickness(f32),
}

struct CombinedVertexData {
    data: Vec<f32>,
    data_types: Vec<(GLenum, u32)>,
}

struct PolygonFillData {
    vertices: Vec<f32>,
    fill_sequence: Vec<GLuint>,
}

struct OperationExtractor {
    data: Vec<RawOperationData>,
}

impl OperationExtractor {
    fn from_svg_vertices(svg_object: &SVG) -> Self {
        let mut extractor = Self { data: Vec::new() };

        for element in svg_object.elements.iter() {
            extractor.data.push(RawOperationData::UpdateLineThickness(
                Style::DEFAULT.stroke_width,
            ));
            extractor.load_element_vertices(&element, &Matrix3x3::IDENTITY3X3, &Style::DEFAULT);
        }

        extractor
    }

    fn load_svg_vertices(&mut self, svg_object: &SVG, transform: &Transform, style: &Style) {
        for element in svg_object.elements.iter() {
            self.load_element_vertices(&element, transform, style);
        }
    }

    fn load_element_vertices(&mut self, element: &Element, transform: &Transform, style: &Style) {
        match element {
            Element::StartTag(start_tag) => {
                self.load_tag_group_vertices(start_tag, transform, style)
            }
            Element::EmptyTag(empty_tag) => {
                self.load_empty_tag_vertices(empty_tag, transform, style)
            }
            Element::EndTag(_) => (),
        }
    }

    fn load_tag_group_vertices(
        &mut self,
        tag_group: &StartTag,
        transform: &Transform,
        style: &Style,
    ) {
        match tag_group {
            StartTag::Group(group) => {
                let new_transform = transform * &group.style.transform;
                self.handle_style_changes(style, &group.style);
                for element in group.elements.iter() {
                    self.load_element_vertices(element, &new_transform, &group.style);
                }
            }
            StartTag::SVG(svg_object) => self.load_svg_vertices(svg_object, transform, style),
        }
    }

    fn load_empty_tag_vertices(
        &mut self,
        empty_tag: &EmptyTag,
        transform: &Transform,
        style: &Style,
    ) {
        match empty_tag {
            EmptyTag::Ellipse(_ellipse) => unimplemented!(),
            EmptyTag::Image(_image) => unimplemented!(),
            EmptyTag::Line(line) => self.load_line_vertices(line, transform, style),
            EmptyTag::Point(point) => self.load_point(point, transform, style),
            EmptyTag::Polygon(polygon) => self.load_polygon_vertices(polygon, transform, style),
            EmptyTag::Polyline(_polyline) => unimplemented!(),
            EmptyTag::Rect(_rect) => unimplemented!(),
        }
    }

    fn load_point(&mut self, point: &Point, transform: &Transform, style: &Style) {
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

        self.handle_style_changes(style, &point.style);

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

    fn load_line_vertices(&mut self, line: &Line, transform: &Transform, style: &Style) {
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

        self.handle_style_changes(style, &line.style);

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
            Some(RawOperationData::DrawVertices(combined_data)) => combined_data,
            _ => {
                self.data
                    .push(RawOperationData::DrawVertices(CombinedVertexData {
                        data: Vec::new(),
                        data_types: Vec::new(),
                    }));
                match self.data.last_mut() {
                    Some(RawOperationData::DrawVertices(combined_data)) => combined_data,
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

    fn load_polygon_vertices(&mut self, polygon: &Polygon, transform: &Transform, style: &Style) {
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
            fill_vertex_data.reserve_exact(polygon.points.len() * (shaders::POS_SIZE + shaders::COLOR_SIZE) as usize);
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
                .reserve_exact(polygon.points.len() * (shaders::POS_SIZE + shaders::COLOR_SIZE) as usize);
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
            self.data
                .push(RawOperationData::FillPolygon(PolygonFillData {
                    vertices: fill_vertex_data,
                    fill_sequence: fill_element_data,
                }));
        }

        if do_outline {
            self.handle_style_changes(style, &polygon.style);

            self.append_combined_data(
                gl::LINE_LOOP,
                &stroke_vertex_data,
                polygon.points.len() as u32,
                false,
            );
        }
    }

    fn handle_style_changes(&mut self, old_style: &Style, new_style: &Style) {
        if old_style.stroke_width != new_style.stroke_width {
            self.data.push(RawOperationData::UpdateLineThickness(
                new_style.stroke_width,
            ));
        }
    }
}

struct CombinedVertexArray {
    array_index: GLuint,
    buffer_index: GLuint,
    data_types: Vec<(GLenum, u32)>,
}

impl CombinedVertexArray {
    unsafe fn draw(&self) {
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
    unsafe fn draw(&self) {
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

enum Operation {
    DrawVertices(CombinedVertexArray),
    FillPolygon(ElementVertexArray),
    UpdateLineThickness(f32),
}

impl Operation {
    fn gen_from_svg(svg_object: &SVG, shaders: &LineShader) -> Vec<Self> {
        let raw_operation_data = OperationExtractor::from_svg_vertices(svg_object);

        let mut operations = Vec::new();

        operations.reserve_exact(raw_operation_data.data.len());

        for operation_data in raw_operation_data.data.into_iter() {
            match operation_data {
                RawOperationData::DrawVertices(combined_data) => {
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

                    operations.push(Operation::DrawVertices(combined_buffer));
                }
                RawOperationData::FillPolygon(polygon_data) => {
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

                    operations.push(Operation::FillPolygon(element_buffer))
                }
                RawOperationData::UpdateLineThickness(thickness) => {
                    operations.push(Operation::UpdateLineThickness(thickness));
                }
            }
        }

        operations
    }

    fn execute(&self, shaders: &LineShader) {
        unsafe {
            match self {
                Operation::DrawVertices(combined_buffer) => {
                    combined_buffer.draw();
                }
                Operation::FillPolygon(element_buffer) => {
                    element_buffer.draw();
                }
                Operation::UpdateLineThickness(thickness) => {
                    shaders.update_line_thickness(*thickness);
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
    shaders: LineShader,
    operation: Vec<Operation>,
}

impl GLRenderer {
    pub fn new(window: Window, gl_ctx: GLContext, object_mgr: &ObjectMgr) -> Result<Self, String> {
        let window_size: [u32; 2] = window.size().into();

        let shaders = LineShader::build()?;

        let mut operations = Vec::new();
        for object in object_mgr.get_objects() {
            operations.extend(Operation::gen_from_svg(&object.svg_inst, &shaders));
        }

        let gl_renderer = Self {
            window,
            _gl_ctx: gl_ctx,
            viewer: GLViewer::new(Vector2D::from(window_size)),
            shaders,
            operation: operations,
        };

        Ok(gl_renderer)
    }

    fn perform_operation(&self, operation: &Operation) {
        operation.execute(&self.shaders);
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
            .update_norm_to_viewer(self.viewer.get_norm_to_viewer());

        for operation in self.operation.iter() {
            self.perform_operation(operation);
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
