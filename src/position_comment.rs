use crate::{ogn_packet::CsvSerializer, utils::split_value_unit};
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
            if !unparsed.is_empty() {
                unparsed.push(part);
            } else if idx == 0 && part.len() == 16 {
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
            } else if idx == 0 && part.len() == 9 && &part[0..3] == "/A=" {
                match part[3..].parse::<u32>().ok() {
                    Some(altitude) => position_comment.altitude = Some(altitude),
                    None => unparsed.push(part),
                }
            } else if idx == 1 && part.len() == 5 && &part[0..2] == "!W" && &part[4..] == "!" {
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
            } else if part.len() == 10 && &part[0..2] == "id" {
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
                if unit == "fpm" {
                    position_comment.climb_rate = value.parse::<i16>().ok();
                } else if unit == "rot" {
                    position_comment.turn_rate = value.parse::<f32>().ok();
                } else if unit == "dB" {
                    position_comment.signal_quality = value.parse::<f32>().ok();
                } else if unit == "kHz" {
                    position_comment.frequency_offset = value.parse::<f32>().ok();
                } else if unit == "e" {
                    position_comment.error = value.parse::<u8>().ok();
                } else if unit == "dBm" {
                    position_comment.signal_power = value.parse::<f32>().ok();
                } else {
                    unparsed.push(part);
                }
            } else if part.len() >= 6 && &part[0..3] == "gps" {
                if let Some((first, second)) = part[3..].split_once('x') {
                    if first.parse::<u8>().is_ok() && second.parse::<u8>().is_ok() {
                        position_comment.gps_quality = Some(part[3..].to_string());
                    } else {
                        unparsed.push(part);
                    }
                } else {
                    unparsed.push(part);
                }
            } else if part.len() >= 3 && &part[0..2] == "FL" {
                if let Ok(flight_level) = part[2..].parse::<f32>() {
                    position_comment.flight_level = Some(flight_level);
                } else {
                    unparsed.push(part);
                }
            } else if part.len() >= 2 && &part[0..1] == "s" {
                if let Ok(software_version) = part[1..].parse::<f32>() {
                    position_comment.software_version = Some(software_version);
                } else {
                    unparsed.push(part);
                }
            } else if part.len() == 3 && &part[0..1] == "h" {
                if part[1..3].chars().all(|c| c.is_ascii_hexdigit()) {
                    position_comment.hardware_version = u8::from_str_radix(&part[1..3], 16).ok();
                } else {
                    unparsed.push(part);
                }
            } else if part.len() == 7 && &part[0..1] == "r" {
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

impl CsvSerializer for PositionComment {
    fn csv_header() -> String {
        "additional_lat,additional_lon,course,speed,altitude,address_type,aircraft_type,is_stealth,is_notrack,address,climb_rate,turn_rate,error,frequency_offset,signal_quality,gps_quality,flight_level,signal_power,software_version,hardware_version,original_address,unparsed".to_string()
    }

    fn to_csv(&self) -> String {
        let (additional_lat, additional_lon) = self
            .additional_precision
            .as_ref()
            .map(|ap| (ap.lat.to_string(), ap.lon.to_string()))
            .unwrap_or_default();

        let course = self.course.map(|val| val.to_string()).unwrap_or_default();
        let speed = self.speed.map(|val| val.to_string()).unwrap_or_default();
        let altitude = self.altitude.map(|val| val.to_string()).unwrap_or_default();
        let (address_type, aircraft_type, is_stealth, is_notrack, address) = self
            .id
            .as_ref()
            .map(|id| {
                (
                    id.address_type.to_string(),
                    id.aircraft_type.to_string(),
                    id.is_stealth.to_string(),
                    id.is_notrack.to_string(),
                    id.address.to_string(),
                )
            })
            .unwrap_or_default();

        let climb_rate = self
            .climb_rate
            .map(|val| val.to_string())
            .unwrap_or_default();
        let turn_rate = self
            .turn_rate
            .map(|val| val.to_string())
            .unwrap_or_default();
        let error = self.error.map(|val| val.to_string()).unwrap_or_default();
        let frequency_offset = self
            .frequency_offset
            .map(|val| val.to_string())
            .unwrap_or_default();
        let signal_quality = self
            .signal_quality
            .map(|val| val.to_string())
            .unwrap_or_default();
        let gps_quality = self.gps_quality.clone().unwrap_or_default();
        let flight_level = self
            .flight_level
            .map(|val| val.to_string())
            .unwrap_or_default();
        let signal_power = self
            .signal_power
            .map(|val| val.to_string())
            .unwrap_or_default();
        let software_version = self
            .software_version
            .map(|val| val.to_string())
            .unwrap_or_default();
        let hardware_version = self.hardware_version.unwrap_or_default();
        let original_address = self.original_address.unwrap_or_default();
        let unparsed = &self
            .unparsed
            .clone()
            .map(|val| val.replace('"', "\"\""))
            .unwrap_or_default();

        format!("{additional_lat},{additional_lon},{course},{speed},{altitude},{address_type},{aircraft_type},{is_stealth},{is_notrack},{address},{climb_rate},{turn_rate},{error},{frequency_offset},{signal_quality},{gps_quality},{flight_level},{signal_power},{software_version},{hardware_version},{original_address},\"{unparsed}\"")
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
fn test_bad_gps() {
    let result: PositionComment =
        "208/063/A=003222 !W97! id06D017DC -395fpm -2.4rot 8.2dB -6.1kHz gps2xFLRD0".into();
    assert_eq!(result.frequency_offset, Some(-6.1));
    assert_eq!(result.gps_quality.is_some(), false);
    assert_eq!(result.unparsed, Some("gps2xFLRD0".to_string()));
}
