use crate::{objects::Object, vector::Vector2D};

pub mod canvas;
pub mod gl;
pub mod triangulation;

/// Virtual camera looking at a canvas containing SVG objects.
/// 
/// Keep in mind that moving the camera in one direction will seems as if everything on the screen
/// is moving in the opposite direction.
pub trait Viewer {
    fn center_on_object(&mut self, object: &Object);

    fn move_to_world_coords(&mut self, new_center: Vector2D<f32>);
    fn move_by_world_coords(&mut self, delta_x: f32, delta_y: f32);
    fn move_by_pixels(&mut self, delta_x: f32, delta_y: f32);

    fn zoom_to(&mut self, new_zoom: f32);
    fn zoom_by(&mut self, zoom_modifier: f32);
}

pub trait Renderer {
    fn get_viewer(&mut self) -> &mut dyn Viewer;

    fn clear(&mut self);

    fn render_objects(&mut self);

    fn present(&mut self);
}
