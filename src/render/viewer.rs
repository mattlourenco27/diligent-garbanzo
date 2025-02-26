use crate::{objects::Object, vector::Vector2D};

pub trait Viewer {
    fn center_on_object(&mut self, object: &Object);

    fn move_to(&mut self, new_center: Vector2D<f32>);
    fn move_by(&mut self, delta_center: Vector2D<f32>);

    fn zoom_to(&mut self, new_zoom: f32);
    fn zoom_by(&mut self, zoom_modifier: f32);
}
