use std::collections::HashMap;

use crate::Receiver;
use aprs_parser::{AprsData, AprsPacket};
use cheap_ruler::{CheapRuler, DistanceUnit};

pub struct DistanceService {
    pub receivers: HashMap<String, Receiver>,
}

impl DistanceService {
    pub fn new() -> Self {
        DistanceService {
            receivers: HashMap::new(),
        }
    }

    pub fn get_distance(&mut self, aprs: &AprsPacket) -> Option<f32> {
        if let AprsData::Position(position) = &aprs.data {
            if aprs.to.call == "OGNSDR" {
                if !self.receivers.contains_key(&aprs.from.call) {
                    let receiver = Receiver {
                        name: aprs.from.call.clone(),
                        position: position.clone(),
                        cheap_ruler: CheapRuler::new(*position.latitude, DistanceUnit::Kilometers),
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
                    let distance = receiver.cheap_ruler.distance(
                        &(*receiver.position.latitude, *receiver.position.longitude).into(),
                        &(*position.latitude, *position.longitude).into(),
                    );
                    return Some(distance);
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
