use std::collections::HashMap;

use crate::Receiver;
use aprs_parser::{AprsData, AprsPacket};
use cheap_ruler::{CheapRuler, DistanceUnit};

pub struct DistanceService {
    receivers: HashMap<String, Receiver>,
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
                    let vorher = self
                        .receivers
                        .get(&aprs.from.call)
                        .unwrap()
                        .position
                        .clone();
                    if position.latitude != vorher.latitude
                        || position.longitude != vorher.longitude
                    {
                        let receiver = Receiver {
                            name: aprs.from.call.clone(),
                            position: position.clone(),
                            cheap_ruler: CheapRuler::new(
                                *position.latitude,
                                DistanceUnit::Kilometers,
                            ),
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
}
