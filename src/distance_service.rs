use std::collections::HashMap;

use crate::Receiver;
use aprs_parser::{AprsData, AprsPacket};
use geo_types::{Coord, Point};

pub struct Relation {
    pub bearing: f64,
    pub distance: f64,
}
pub struct DistanceService {
    pub receivers: HashMap<String, Receiver>,
}

impl DistanceService {
    pub fn new() -> Self {
        DistanceService {
            receivers: HashMap::new(),
        }
    }

    pub fn get_relation(&mut self, aprs: &AprsPacket) -> Option<Relation> {
        if let AprsData::Position(position) = &aprs.data {
            let position = Coord {
                x: *position.longitude as f64,
                y: *position.latitude as f64,
            };
            if !aprs.from.call.starts_with("RND")
                && ["APRS", "OGNSDR"].contains(&aprs.to.call.as_str())
                && aprs.via.iter().last().unwrap().call.starts_with("GLIDERN")
            {
                if !self.receivers.contains_key(&aprs.from.call) {
                    let receiver = Receiver::new(aprs.from.call.clone(), position);
                    self.receivers.insert(aprs.from.call.clone(), receiver);
                } else {
                    let position_old = self.receivers.get(&aprs.from.call).unwrap().position;
                    if position != position_old {
                        let receiver = Receiver::new(aprs.from.call.clone(), position);
                        self.receivers.insert(aprs.from.call.clone(), receiver);
                    }
                }
            } else if let Some(last_via) = aprs.via.iter().last() {
                if let Some(receiver) = self.receivers.get(&last_via.call) {
                    return Some(Self::calculate_bearing_and_distance(receiver, &position));
                }
            }
        }
        None
    }

    pub fn calculate_bearing_and_distance(receiver: &Receiver, position: &Coord) -> Relation {
        let p1: Point<f64> = receiver.position.into();
        let p2: Point<f64> = (*position).into();
        let bearing = receiver.cheap_ruler.bearing(&p1, &p2);
        let distance = receiver.cheap_ruler.distance(&p1, &p2);
        Relation { bearing, distance }
    }

    pub fn get_normalized_quality(distance: f64, signal_quality: f64) -> Option<f64> {
        match distance > 0.0 {
            true => Some(signal_quality + 20.0 * (distance / 10_000.0).log10()),
            false => None,
        }
    }
}
