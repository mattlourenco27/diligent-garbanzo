use sdl2::{pixels::Color, render::WindowCanvas, video::Window, IntegerOrSdlError};

use crate::objects::{svg::{Element, EmptyTag, Point, StartTag, Style, SVG}, ObjectMgr};

pub struct CanvasRenderer<'a> {
    canvas: WindowCanvas,
    object_mgr: &'a ObjectMgr,
    style: Style,
}

impl<'a> CanvasRenderer<'a> {
    pub fn new(window: Window, object_mgr: &'a ObjectMgr) -> Result<Self, IntegerOrSdlError> {
        let mut canvas = window.into_canvas().present_vsync().build()?;
        let style = Style::DEFAULT;

        canvas.set_draw_color(style.fill_color);

        Ok(Self { canvas , object_mgr, style: Style::DEFAULT})
    }

    pub fn clear(&mut self) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
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
        self.canvas.set_draw_color(Color::RGB(50, 50, 50));
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
            EmptyTag::Line(_line) => unimplemented!(),
            EmptyTag::Point(point) => self.render_point(point),
            EmptyTag::Polygon(_polygon) => unimplemented!(),
            EmptyTag::Polyline(_polyline) => unimplemented!(),
            EmptyTag::Rect(_rect) => unimplemented!(),
        }
    }

    fn render_point(&mut self, point: &Point) {
        self.canvas
            .draw_point(sdl2::rect::Point::new(
                (point.position[0] * 800.0) as i32,
                (point.position[1] * 600.0) as i32,
            ))
            .unwrap();
    }
}
