use flat_projection::{FlatPoint, FlatProjection};

pub struct Receiver {
    pub name: String,
    pub position: (f64, f64),
    pub flat_projection: FlatProjection<f64>,
    pub flat_point: FlatPoint<f64>,
}

impl Receiver {
    pub fn new(name: String, position: (f64, f64)) -> Self {
        let flat_projection = FlatProjection::new(position.0, position.1);
        let flat_point = flat_projection.project(position.0, position.1);
        Receiver {
            name,
            position,
            flat_projection,
            flat_point,
        }
    }
}
