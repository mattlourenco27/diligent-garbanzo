use sdl2::{pixels::Color, render::WindowCanvas, video::Window, IntegerOrSdlError};

use crate::{
    objects::{
        svg::{Element, EmptyTag, Line, Point, StartTag, SVG},
        ObjectMgr,
    },
    vector::Vector2D,
    viewer::Viewer,
};

pub struct Renderer<'a> {
    canvas: WindowCanvas,
    object_mgr: &'a ObjectMgr,
    pub viewer: Viewer,
}

impl<'a> Renderer<'a> {
    pub fn new(window: Window, object_mgr: &'a ObjectMgr) -> Result<Self, IntegerOrSdlError> {
        let window_size: [u32; 2] = window.size().into();
        Ok(Self {
            canvas: window.into_canvas().present_vsync().build()?,
            object_mgr,
            viewer: Viewer::new(Vector2D::from(window_size)),
        })
    }

    pub fn clear(&mut self) {
        self.canvas.set_draw_color(Color::WHITE);
        self.canvas.clear();
    }

    pub fn render_objects(&mut self) {
        for object in self.object_mgr.get_objects() {
            self.render_svg(&object.svg_inst);
        }
    }

    pub fn present(&mut self) {
        self.canvas.present();
    }

    fn render_svg(&mut self, svg_object: &SVG) {
        for element in svg_object.elements.iter() {
            self.render_element(element);
        }
    }

    fn render_element(&mut self, element: &Element) {
        match element {
            Element::StartTag(start_tag) => self.render_tag_group(start_tag),
            Element::EmptyTag(empty_tag) => self.render_empty_tag(empty_tag),
            Element::EndTag(_) => (),
        }
    }

    fn render_tag_group(&mut self, tag_group: &StartTag) {
        match tag_group {
            StartTag::Group(group) => {
                for element in group.elements.iter() {
                    self.render_element(element);
                }
            }
            StartTag::SVG(svg_object) => self.render_svg(svg_object),
        }
    }

    fn render_empty_tag(&mut self, empty_tag: &EmptyTag) {
        match empty_tag {
            EmptyTag::Ellipse(_ellipse) => unimplemented!(),
            EmptyTag::Image(_image) => unimplemented!(),
            EmptyTag::Line(line) => self.render_line(line),
            EmptyTag::Point(point) => self.render_point(point),
            EmptyTag::Polygon(_polygon) => unimplemented!(),
            EmptyTag::Polyline(_polyline) => unimplemented!(),
            EmptyTag::Rect(_rect) => unimplemented!(),
        }
    }

    fn render_point(&mut self, point: &Point) {
        self.canvas.set_draw_color(point.style.fill_color);

        let draw_position = self.viewer.norm_to_viewer(&point.position);
        self.canvas
            .draw_fpoint(sdl2::rect::FPoint::new(
                draw_position[0] as f32,
                draw_position[1] as f32,
            ))
            .unwrap();
    }

    fn render_line(&mut self, line: &Line) {
        self.canvas.set_draw_color(line.style.fill_color);

        let from_position = self.viewer.norm_to_viewer(&line.from);
        let to_position = self.viewer.norm_to_viewer(&line.to);
        self.canvas
            .draw_fline(
                sdl2::rect::FPoint::new(from_position[0] as f32, from_position[1] as f32),
                sdl2::rect::FPoint::new(to_position[0] as f32, to_position[1] as f32),
            )
            .unwrap();
    }
}
