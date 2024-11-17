use num_traits::ConstZero;

use crate::{
    matrix::Matrix3x3,
    objects::Object,
    vector::{Vector2D, Vector3D},
};

pub struct Viewer {
    window_size: Vector2D<u32>,
    center: Vector2D<f64>,
    zoom: f64,
    norm_to_self_transform: Matrix3x3<f64>,
}

impl Viewer {
    pub fn new(window_size: Vector2D<u32>) -> Self {
        let mut ret = Self {
            window_size,
            center: Vector2D::ZERO,
            zoom: 1.0,
            norm_to_self_transform: Matrix3x3::IDENTITY3X3,
        };
        ret.regenerate_norm_to_self_transform();
        ret
    }

    pub fn center_on_object(&mut self, object: &Object) {
        let object_radius = object.svg_inst.dimension.clone() * 0.5;
        self.center[0] = object.position[0] + object_radius[0];
        self.center[1] = object.position[1] + object_radius[1];

        let zoom_x = self.window_size[0] as f64 / object.svg_inst.dimension[0];
        let zoom_y = self.window_size[1] as f64 / object.svg_inst.dimension[1];

        self.zoom = std::cmp::min_by(zoom_x, zoom_y, |x, y| x.partial_cmp(y).unwrap());
        self.regenerate_norm_to_self_transform();
    }

    pub fn move_to(&mut self, new_center: Vector2D<f64>) {
        self.center = new_center;
        self.regenerate_norm_to_self_transform();
    }

    pub fn move_by(&mut self, delta_center: Vector2D<f64>) {
        self.center += delta_center * (1.0 / self.zoom);
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
        let transformed = Vector3D::from_vector(position) * &self.norm_to_self_transform;
        Vector2D::from_vector(&transformed)
    }

    fn regenerate_norm_to_self_transform(&mut self) {
        // Translate to viewer position
        let mut position_matrix = Matrix3x3::IDENTITY3X3;
        position_matrix[2][0] = -self.center[0];
        position_matrix[2][1] = -self.center[1];

        // Zoom the appropriate amount
        let mut zoom_matrix = Matrix3x3::IDENTITY3X3;
        zoom_matrix[0][0] = self.zoom;
        zoom_matrix[1][1] = self.zoom;

        // Move origin to center of the viewer
        let mut center_matrix = Matrix3x3::IDENTITY3X3;
        center_matrix[2][0] = self.window_size[0] as f64 / 2.0;
        center_matrix[2][1] = self.window_size[1] as f64 / 2.0;

        self.norm_to_self_transform = &position_matrix * &zoom_matrix * &center_matrix;
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        objects::{svg::SVG, Object},
        vector::{Vector2D, Vector3D},
    };

    use super::Viewer;

    fn new_viewer() -> Viewer {
        Viewer::new(Vector2D::from([100, 100]))
    }

    #[test]
    fn init_at_origin() {
        let viewer = new_viewer();
        assert_eq!(viewer.center, Vector2D::from([0.0, 0.0]));
    }

    #[test]
    fn pixels_at_viewer_center_map_to_the_screen_center() {
        let viewer = Viewer::new(Vector2D::from([100, 100]));
        assert_eq!(
            viewer.norm_to_viewer(&viewer.center),
            Vector2D::from([50.0, 50.0])
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
        const ZOOM_AMOUNT: f64 = 1.0;

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
        const ZOOM_AMOUNT: f64 = 3.77;

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

        assert_eq!(viewer.center, Vector2D::from([14.0, 7.0]));
    }

    #[test]
    fn viewer_zooms_to_given_object_size() {
        todo!()
    }

    #[test]
    fn viewer_shouldnt_panic_when_object_size_is_zero() {
        todo!()
    }

    #[test]
    fn viewer_centers_itself_on_position_to_move_to() {
        todo!()
    }

    #[test]
    fn viewer_moves_by_amount_specified_multiplied_by_zoom() {
        todo!()
    }
}
