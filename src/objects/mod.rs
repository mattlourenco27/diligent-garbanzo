use svg::SVG;

use crate::vector::Vector3D;

pub mod svg;

pub struct Object {
    pub position: Vector3D<f64>,
    pub svg_inst: SVG,
}

impl From<SVG> for Object {
    fn from(value: SVG) -> Self {
        Self {
            position: [0.0, 0.0, 0.0].into(),
            svg_inst: value,
        }
    }
}

pub struct ObjectMgr {
    objects: Vec<Object>,
}

impl ObjectMgr {
    pub fn new() -> ObjectMgr {
        ObjectMgr {
            objects: Vec::new(),
        }
    }

    pub fn get_objects(&self) -> &[Object] {
        &self.objects
    }

    pub fn add_object(&mut self, object: Object) {
        self.objects.push(object);
    }
}
