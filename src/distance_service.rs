use std::collections::HashMap;

use crate::Receiver;
use aprs_parser::{AprsData, AprsPacket};
use cheap_ruler::{CheapRuler, DistanceUnit};

pub struct Relation {
    pub bearing: f32,
    pub distance: f32,
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
            if !aprs.from.call.starts_with("RND")
                && ["APRS", "OGNSDR"].contains(&aprs.to.call.as_str())
                && aprs.via.iter().last().unwrap().call.starts_with("GLIDERN")
            {
                if !self.receivers.contains_key(&aprs.from.call) {
                    let receiver = Receiver {
                        name: aprs.from.call.clone(),
                        position: position.clone(),
                        cheap_ruler: CheapRuler::new(*position.latitude, DistanceUnit::Meters),
                    };
                    self.receivers.insert(aprs.from.call.clone(), receiver);
                } else {
                    let position_old = self
                        .receivers
                        .get(&aprs.from.call)
                        .unwrap()
                        .position
                        .clone();
                    if position.latitude != position_old.latitude
                        || position.longitude != position_old.longitude
                    {
                        let receiver = Receiver {
                            name: aprs.from.call.clone(),
                            position: position.clone(),
                            cheap_ruler: CheapRuler::new(*position.latitude, DistanceUnit::Meters),
                        };
                        self.receivers.insert(aprs.from.call.clone(), receiver);
                    }
                }
            } else {
                let receiver_name = aprs.via.iter().last().unwrap().call.clone();
                if self.receivers.contains_key(&receiver_name) {
                    let receiver = self.receivers.get(&receiver_name).unwrap();
                    let p1 = (*receiver.position.latitude, *receiver.position.longitude).into();
                    let p2 = (*position.latitude, *position.longitude).into();
                    let bearing = receiver.cheap_ruler.bearing(&p1, &p2);
                    let distance = receiver.cheap_ruler.distance(&p1, &p2);
                    return Some(Relation { bearing, distance });
                }
            }
        }
        None
    }

    pub fn get_normalized_quality(distance: f32, signal_quality: f32) -> Option<f32> {
        match distance > 0.0 {
            true => Some(signal_quality + 20.0 * (distance / 10_000.0).log10()),
            false => None,
        }
    }
}
