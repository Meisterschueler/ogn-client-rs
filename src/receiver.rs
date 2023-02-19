use cheap_ruler::{CheapRuler, DistanceUnit};
use geo_types::Coord;

pub struct Receiver {
    pub name: String,
    pub position: Coord,
    pub cheap_ruler: CheapRuler<f64>,
}

impl Receiver {
    pub fn new(name: String, position: Coord) -> Self {
        let cheap_ruler = CheapRuler::new(position.y, DistanceUnit::Meters);
        Receiver {
            name,
            position,
            cheap_ruler,
        }
    }
}
