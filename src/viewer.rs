use crate::{
    matrix::{Matrix3x3, StaticMatrix},
    vector::Vector2D,
};

pub struct Viewer {
    transform: Matrix3x3<f64>,
}

impl Viewer {
    pub fn new() -> Self {
        Self {
            transform: Matrix3x3::IDENTITY3X3
        }
    }

    pub fn move_to(&mut self, new_position: &Vector2D<f64>) {
        self.transform[2][0] = self.transform[0][0] * new_position[0];
        self.transform[2][1] = self.transform[1][1] * new_position[1];
    }

    pub fn zoom_to(&mut self, new_zoom: &Vector2D<f64>) {
        self.transform[2][0] *= new_zoom[0];
        self.transform[2][1] *= new_zoom[1];
        self.transform[0][0] = new_zoom[0];
        self.transform[1][1] = new_zoom[1];
    }

    pub fn transform_position(&self, position: &Vector2D<f64>) -> Vector2D<f64> {
        let mut transformed = StaticMatrix::from([[position[0], position[1], 1.0]]);
        transformed *= &self.transform;
        [transformed[0][0], transformed[0][1]].into()
    }
}
