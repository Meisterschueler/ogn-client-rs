use crate::{ogn_packet::OGNPacketPosition, Receiver};
use std::collections::HashMap;
#[derive(Debug, Copy, Clone, PartialEq)]
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

    pub fn get_relation(&mut self, packet: &OGNPacketPosition) -> Option<Relation> {
        if packet.src_call.starts_with("RND") {
            return None;
        }

        let position = (*packet.aprs.longitude, *packet.aprs.latitude);
        if packet.receiver.starts_with("GLIDERN") {
            if !self.receivers.contains_key(&packet.src_call) {
                let receiver = Receiver::new(packet.src_call.clone(), position);
                self.receivers.insert(packet.src_call.clone(), receiver);
            } else {
                let position_old = self.receivers.get(&packet.src_call).unwrap().position;
                if position != position_old {
                    let receiver = Receiver::new(packet.src_call.clone(), position);
                    self.receivers.insert(packet.src_call.clone(), receiver);
                }
            }
        } else if let Some(receiver) = self.receivers.get(&packet.receiver) {
            return Some(Self::calculate_bearing_and_distance(receiver, &position));
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
