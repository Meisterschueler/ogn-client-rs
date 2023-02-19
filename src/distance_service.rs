use std::collections::HashMap;

use crate::Receiver;
use aprs_parser::{AprsData, AprsPacket};

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
            let position = (*position.longitude as f64, *position.latitude as f64);
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

    pub fn calculate_bearing_and_distance(receiver: &Receiver, position: &(f64, f64)) -> Relation {
        let flat_point = receiver.flat_projection.project(position.0, position.1);
        let bearing = flat_point.bearing(&receiver.flat_point);
        let distance = flat_point.distance(&receiver.flat_point) * 1000.0;

        Relation {
            bearing: if bearing < 0.0 {
                bearing + 360.0
            } else {
                bearing
            },
            distance,
        }
    }

    pub fn get_normalized_quality(distance: f64, signal_quality: f64) -> Option<f64> {
        match distance > 0.0 {
            true => Some(signal_quality + 20.0 * (distance / 10_000.0).log10()),
            false => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bearing_and_distance() {
        let receiver = Receiver::new("TestReceiver".to_string(), (13.0, 52.0));

        let position = (13.0, 51.0);
        let relation = DistanceService::calculate_bearing_and_distance(&receiver, &position);
        assert_eq!(relation.bearing, 0.0);
        assert_eq!(relation.distance, 111267.35329292723);

        let position = (12.0, 52.0);
        let relation = DistanceService::calculate_bearing_and_distance(&receiver, &position);
        assert_eq!(relation.bearing, 90.0);
        assert_eq!(relation.distance, 68678.01607929853);

        let position = (13.0, 53.0);
        let relation = DistanceService::calculate_bearing_and_distance(&receiver, &position);
        assert_eq!(relation.bearing, 180.0);

        let position = (14.0, 52.0);
        let relation = DistanceService::calculate_bearing_and_distance(&receiver, &position);
        assert_eq!(relation.bearing, 270.0);
    }
}
