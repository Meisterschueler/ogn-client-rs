use aprs_parser::AprsPosition;
use cheap_ruler::CheapRuler;

pub struct Receiver {
    pub name: String,
    pub position: AprsPosition,
    pub cheap_ruler: CheapRuler<f32>,
}
