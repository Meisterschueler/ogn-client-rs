use std::collections::HashMap;

use crate::{ogn_packet::ElementGetter, utils::split_value_unit};
#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct AdditionalPrecision {
    pub lat: u8,
    pub lon: u8,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ID {
    pub address_type: u8,
    pub aircraft_type: u8,
    pub is_stealth: bool,
    pub is_notrack: bool,
    pub address: u32,
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct PositionComment {
    pub course: Option<u16>,
    pub speed: Option<u16>,
    pub altitude: Option<u32>,
    pub additional_precision: Option<AdditionalPrecision>,
    pub id: Option<ID>,
    pub climb_rate: Option<i16>,
    pub turn_rate: Option<f32>,
    pub signal_quality: Option<f32>,
    pub error: Option<u8>,
    pub frequency_offset: Option<f32>,
    pub gps_quality: Option<String>,
    pub flight_level: Option<f32>,
    pub signal_power: Option<f32>,
    pub software_version: Option<f32>,
    pub hardware_version: Option<u8>,
    pub original_address: Option<u32>,
    pub unparsed: Option<String>,
}

impl From<&str> for PositionComment {
    fn from(s: &str) -> Self {
        let mut position_comment = PositionComment {
            ..Default::default()
        };
        let mut unparsed: Vec<_> = vec![];
        for (idx, part) in s.split_ascii_whitespace().enumerate() {
            // The first part can be course + speed + altitude: ccc/sss/A=aaaaaa
            // ccc: course in degrees 0-360
            // sss: speed in km/h
            // aaaaaa: altitude in feet
            if idx == 0 && part.len() == 16 && position_comment.course.is_none() {
                let subparts = part.split('/').collect::<Vec<_>>();
                let course = subparts[0].parse::<u16>().ok();
                let speed = subparts[1].parse::<u16>().ok();
                let altitude = if &subparts[2][0..2] == "A=" {
                    subparts[2][2..].parse::<u32>().ok()
                } else {
                    None
                };
                if course.is_some()
                    && course.unwrap() <= 360
                    && speed.is_some()
                    && altitude.is_some()
                {
                    position_comment.course = course;
                    position_comment.speed = speed;
                    position_comment.altitude = altitude;
                } else {
                    unparsed.push(part);
                }
            // ... or just the altitude: /A=aaaaaa
            // aaaaaa: altitude in feet
            } else if idx == 0
                && part.len() == 9
                && &part[0..3] == "/A="
                && position_comment.altitude.is_none()
            {
                match part[3..].parse::<u32>().ok() {
                    Some(altitude) => position_comment.altitude = Some(altitude),
                    None => unparsed.push(part),
                }
            // The second part can be the additional precision: !Wab!
            // a: additional latitude precision
            // b: additional longitude precision
            } else if idx == 1
                && part.len() == 5
                && &part[0..2] == "!W"
                && &part[4..] == "!"
                && position_comment.additional_precision.is_none()
            {
                let add_lat = part[2..3].parse::<u8>().ok();
                let add_lon = part[3..4].parse::<u8>().ok();
                match (add_lat, add_lon) {
                    (Some(add_lat), Some(add_lon)) => {
                        position_comment.additional_precision = Some(AdditionalPrecision {
                            lat: add_lat,
                            lon: add_lon,
                        })
                    }
                    _ => unparsed.push(part),
                }
            // idXXYYYYYY is for the ID
            // YYYYYY: 24 bit address in hex digits
            // XX in hex digits encodes stealth mode, no-tracking flag and address type
            // XX to binary-> STttttaa
            // S: stealth flag
            // T: no-tracking flag
            // tttt: aircraft type
            // aa: address type
            } else if part.len() == 10 && &part[0..2] == "id" && position_comment.id.is_none() {
                if let (Some(detail), Some(address)) = (
                    u8::from_str_radix(&part[2..4], 16).ok(),
                    u32::from_str_radix(&part[4..10], 16).ok(),
                ) {
                    let address_type = detail & 0b0000_0011;
                    let aircraft_type = (detail & 0b0011_1100) >> 2;
                    let is_notrack = (detail & 0b0100_0000) != 0;
                    let is_stealth = (detail & 0b1000_0000) != 0;
                    position_comment.id = Some(ID {
                        address_type,
                        aircraft_type,
                        is_notrack,
                        is_stealth,
                        address,
                    });
                } else {
                    unparsed.push(part);
                }
            } else if let Some((value, unit)) = split_value_unit(part) {
                if unit == "fpm" && position_comment.climb_rate.is_none() {
                    position_comment.climb_rate = value.parse::<i16>().ok();
                } else if unit == "rot" && position_comment.turn_rate.is_none() {
                    position_comment.turn_rate = value.parse::<f32>().ok();
                } else if unit == "dB" && position_comment.signal_quality.is_none() {
                    position_comment.signal_quality = value.parse::<f32>().ok();
                } else if unit == "kHz" && position_comment.frequency_offset.is_none() {
                    position_comment.frequency_offset = value.parse::<f32>().ok();
                } else if unit == "e" && position_comment.error.is_none() {
                    position_comment.error = value.parse::<u8>().ok();
                } else if unit == "dBm" && position_comment.signal_power.is_none() {
                    position_comment.signal_power = value.parse::<f32>().ok();
                } else {
                    unparsed.push(part);
                }
            // Gps precision: gpsAxB
            // A: integer
            // B: integer
            } else if part.len() >= 6
                && &part[0..3] == "gps"
                && position_comment.gps_quality.is_none()
            {
                if let Some((first, second)) = part[3..].split_once('x') {
                    if first.parse::<u8>().is_ok() && second.parse::<u8>().is_ok() {
                        position_comment.gps_quality = Some(part[3..].to_string());
                    } else {
                        unparsed.push(part);
                    }
                } else {
                    unparsed.push(part);
                }
            // Flight level: FLxx.yy
            // xx.yy: float value for flight level
            } else if part.len() >= 3
                && &part[0..2] == "FL"
                && position_comment.flight_level.is_none()
            {
                if let Ok(flight_level) = part[2..].parse::<f32>() {
                    position_comment.flight_level = Some(flight_level);
                } else {
                    unparsed.push(part);
                }
            // Software version: sXX.YY
            // XX.YY: float value for software version
            } else if part.len() >= 2
                && &part[0..1] == "s"
                && position_comment.software_version.is_none()
            {
                if let Ok(software_version) = part[1..].parse::<f32>() {
                    position_comment.software_version = Some(software_version);
                } else {
                    unparsed.push(part);
                }
            // Hardware version: hXX
            // XX: hexadecimal value for hardware version
            } else if part.len() == 3
                && &part[0..1] == "h"
                && position_comment.hardware_version.is_none()
            {
                if part[1..3].chars().all(|c| c.is_ascii_hexdigit()) {
                    position_comment.hardware_version = u8::from_str_radix(&part[1..3], 16).ok();
                } else {
                    unparsed.push(part);
                }
            // Original address: rXXXXXX
            // XXXXXX: hex digits for 24 bit address
            } else if part.len() == 7
                && &part[0..1] == "r"
                && position_comment.original_address.is_none()
            {
                if part[1..7].chars().all(|c| c.is_ascii_hexdigit()) {
                    position_comment.original_address = u32::from_str_radix(&part[1..7], 16).ok();
                } else {
                    unparsed.push(part);
                }
            } else {
                unparsed.push(part);
            }
        }
        position_comment.unparsed = if !unparsed.is_empty() {
            Some(unparsed.join(" "))
        } else {
            None
        };
        position_comment
    }
}

impl ElementGetter for PositionComment {
    fn get_elements(&self) -> HashMap<&str, String> {
        let mut elements: HashMap<&str, String> = HashMap::new();

        if let Some(course) = self.course {
            elements.insert("course", course.to_string());
        };
        if let Some(speed) = self.speed {
            elements.insert("speed", speed.to_string());
        };
        if let Some(altitude) = self.altitude {
            elements.insert("altitude", altitude.to_string());
        };
        if let Some(additional_precision) = &self.additional_precision {
            elements.insert("additional_lat", additional_precision.lat.to_string());
            elements.insert("additional_lon", additional_precision.lon.to_string());
        };
        if let Some(id) = &self.id {
            elements.insert("address_type", id.address_type.to_string());
            elements.insert("aircraft_type", id.aircraft_type.to_string());
            elements.insert("is_stealth", id.is_stealth.to_string());
            elements.insert("is_notrack", id.is_notrack.to_string());
            elements.insert("address", id.address.to_string());
        };

        if let Some(climb_rate) = self.climb_rate {
            elements.insert("climb_rate", climb_rate.to_string());
        };
        if let Some(turn_rate) = self.turn_rate {
            elements.insert("turn_rate", turn_rate.to_string());
        };
        if let Some(signal_quality) = self.signal_quality {
            elements.insert("signal_quality", signal_quality.to_string());
        };
        if let Some(error) = self.error {
            elements.insert("error", error.to_string());
        };
        if let Some(frequency_offset) = self.frequency_offset {
            elements.insert("frequency_offset", frequency_offset.to_string());
        };
        if let Some(gps_quality) = &self.gps_quality {
            elements.insert("gps_quality", gps_quality.clone());
        };
        if let Some(flight_level) = self.flight_level {
            elements.insert("flight_level", flight_level.to_string());
        };
        if let Some(signal_power) = self.signal_power {
            elements.insert("signal_power", signal_power.to_string());
        };
        if let Some(software_version) = self.software_version {
            elements.insert("software_version", software_version.to_string());
        };
        if let Some(hardware_version) = self.hardware_version {
            elements.insert("hardware_version", hardware_version.to_string());
        };
        if let Some(original_address) = self.original_address {
            elements.insert("original_address", original_address.to_string());
        };
        if let Some(unparsed) = &self.unparsed {
            elements.insert("unparsed", unparsed.clone());
        };

        elements
    }
}

#[test]
fn test_flr() {
    let result: PositionComment = "255/045/A=003399 !W03! id06DDFAA3 -613fpm -3.9rot 22.5dB 7e -7.0kHz gps3x7 s7.07 h41 rD002F8".into();
    assert_eq!(
        result,
        PositionComment {
            course: Some(255),
            speed: Some(45),
            altitude: Some(3399),
            additional_precision: Some(AdditionalPrecision { lat: 0, lon: 3 }),
            id: Some(ID {
                address_type: 2,
                aircraft_type: 1,
                is_stealth: false,
                is_notrack: false,
                address: u32::from_str_radix("DDFAA3", 16).unwrap()
            }),
            climb_rate: Some(-613),
            turn_rate: Some(-3.9),
            signal_quality: Some(22.5),
            error: Some(7),
            frequency_offset: Some(-7.0),
            gps_quality: Some("3x7".into()),
            software_version: Some(7.07),
            hardware_version: Some(65),
            original_address: u32::from_str_radix("D002F8", 16).ok(),
            ..Default::default()
        }
    );
}

#[test]
fn test_trk() {
    let result: PositionComment =
        "200/073/A=126433 !W05! id15B50BBB +4237fpm +2.2rot FL1267.81 10.0dB 19e +23.8kHz gps36x55"
            .into();
    assert_eq!(
        result,
        PositionComment {
            course: Some(200),
            speed: Some(73),
            altitude: Some(126433),
            additional_precision: Some(AdditionalPrecision { lat: 0, lon: 5 }),
            id: Some(ID {
                address_type: 1,
                aircraft_type: 5,
                is_stealth: false,
                is_notrack: false,
                address: u32::from_str_radix("B50BBB", 16).unwrap()
            }),
            climb_rate: Some(4237),
            turn_rate: Some(2.2),
            signal_quality: Some(10.0),
            error: Some(19),
            frequency_offset: Some(23.8),
            gps_quality: Some("36x55".into()),
            flight_level: Some(1267.81),
            signal_power: None,
            software_version: None,
            hardware_version: None,
            original_address: None,
            unparsed: None
        }
    );
}

#[test]
fn test_trk2() {
    let result: PositionComment = "000/000/A=002280 !W59! id07395004 +000fpm +0.0rot FL021.72 40.2dB -15.1kHz gps9x13 +15.8dBm".into();
    assert_eq!(
        result,
        PositionComment {
            course: Some(0),
            speed: Some(0),
            altitude: Some(2280),
            additional_precision: Some(AdditionalPrecision { lat: 5, lon: 9 }),
            id: Some(ID {
                address_type: 3,
                aircraft_type: 1,
                is_stealth: false,
                is_notrack: false,
                address: u32::from_str_radix("395004", 16).unwrap()
            }),
            climb_rate: Some(0),
            turn_rate: Some(0.0),
            signal_quality: Some(40.2),
            frequency_offset: Some(-15.1),
            gps_quality: Some("9x13".into()),
            flight_level: Some(21.72),
            signal_power: Some(15.8),
            ..Default::default()
        }
    );
}

#[test]
fn test_trk2_different_order() {
    // Check if order doesn't matter
    let result: PositionComment = "000/000/A=002280 !W59! -15.1kHz id07395004 +15.8dBm +0.0rot +000fpm FL021.72 40.2dB gps9x13".into();
    assert_eq!(
        result,
        PositionComment {
            course: Some(0),
            speed: Some(0),
            altitude: Some(2280),
            additional_precision: Some(AdditionalPrecision { lat: 5, lon: 9 }),
            id: Some(ID {
                address_type: 3,
                aircraft_type: 1,
                is_stealth: false,
                is_notrack: false,
                address: u32::from_str_radix("395004", 16).unwrap()
            }),
            climb_rate: Some(0),
            turn_rate: Some(0.0),
            signal_quality: Some(40.2),
            frequency_offset: Some(-15.1),
            gps_quality: Some("9x13".into()),
            flight_level: Some(21.72),
            signal_power: Some(15.8),
            ..Default::default()
        }
    );
}

#[test]
fn test_bad_gps() {
    let result: PositionComment =
        "208/063/A=003222 !W97! id06D017DC -395fpm -2.4rot 8.2dB -6.1kHz gps2xFLRD0".into();
    assert_eq!(result.frequency_offset, Some(-6.1));
    assert_eq!(result.gps_quality.is_some(), false);
    assert_eq!(result.unparsed, Some("gps2xFLRD0".to_string()));
}
