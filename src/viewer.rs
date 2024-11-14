use num_traits::ConstZero;

use crate::{
    matrix::Matrix3x3,
    vector::{Vector2D, Vector3D},
};

pub struct Viewer {
    position: Vector2D<f64>,
    zoom: f64,
    norm_to_self_transform: Matrix3x3<f64>,
}

impl Viewer {
    pub fn new() -> Self {
        Self {
            position: Vector2D::ZERO,
            zoom: 1.0,
            norm_to_self_transform: Matrix3x3::IDENTITY3X3,
        }
    }

    pub fn move_to(&mut self, new_position: Vector2D<f64>) {
        self.position = new_position;
        self.regenerate_norm_to_self_transform();
    }

    pub fn move_by(&mut self, delta_position: &Vector2D<f64>) {
        self.position += delta_position;
        self.regenerate_norm_to_self_transform();
    }

    pub fn zoom_to(&mut self, new_zoom: f64) {
        self.zoom = new_zoom;
        self.regenerate_norm_to_self_transform();
    }

    pub fn zoom_by(&mut self, zoom_modifier: f64) {
        self.zoom *= zoom_modifier;
        self.regenerate_norm_to_self_transform();
    }

    pub fn norm_to_viewer(&self, position: &Vector2D<f64>) -> Vector2D<f64> {
        let mut transformed = Vector3D::from([position[0], position[1], 1.0]);
        transformed *= &self.norm_to_self_transform;
        [transformed[0], transformed[1]].into()
    }

    fn regenerate_norm_to_self_transform(&mut self)
    {
        let mut position_matrix = Matrix3x3::IDENTITY3X3;
        position_matrix[2][0] = -self.position[0];
        position_matrix[2][1] = -self.position[1];

        let mut zoom_matrix = Matrix3x3::IDENTITY3X3;
        zoom_matrix[0][0] = self.zoom;
        zoom_matrix[1][1] = self.zoom;

        self.norm_to_self_transform = position_matrix * zoom_matrix;
    }
}
