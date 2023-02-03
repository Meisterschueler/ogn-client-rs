#[derive(Debug, PartialEq, Eq)]
pub struct AdditionalPrecision {
    pub lat: u8,
    pub lon: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ID {
    pub address_type: u8,
    pub aircraft_type: u8,
    pub is_stealth: bool,
    pub is_notrack: bool,
    pub address: u32,
}

#[derive(Debug, PartialEq, Default)]
pub struct OGNComment {
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
    pub hardware_version: Option<u16>,
    pub real_id: Option<u32>,
    pub comment: Option<String>,
}

impl From<&str> for OGNComment {
    fn from(s: &str) -> Self {
        let mut ogn_comment = OGNComment {
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
                    ogn_comment.course = course;
                    ogn_comment.speed = speed;
                    ogn_comment.altitude = altitude;
                } else {
                    unparsed.push(part);
                }
            } else if idx == 0 && part.len() == 9 && &part[0..3] == "/A=" {
                match part[3..].parse::<u32>().ok() {
                    Some(altitude) => ogn_comment.altitude = Some(altitude),
                    None => unparsed.push(part),
                }
            } else if idx == 1 && part.len() == 5 && &part[0..2] == "!W" && &part[4..] == "!" {
                let add_lat = part[2..3].parse::<u8>().ok();
                let add_lon = part[3..4].parse::<u8>().ok();
                match (add_lat, add_lon) {
                    (Some(add_lat), Some(add_lon)) => {
                        ogn_comment.additional_precision = Some(AdditionalPrecision {
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
                    ogn_comment.id = Some(ID {
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
                    ogn_comment.climb_rate = value.parse::<i16>().ok();
                } else if unit == "rot" {
                    ogn_comment.turn_rate = value.parse::<f32>().ok();
                } else if unit == "dB" {
                    ogn_comment.signal_quality = value.parse::<f32>().ok();
                } else if unit == "kHz" {
                    ogn_comment.frequency_offset = value.parse::<f32>().ok();
                } else if unit == "e" {
                    ogn_comment.error = value.parse::<u8>().ok();
                } else if unit == "dBm" {
                    ogn_comment.signal_power = value.parse::<f32>().ok();
                } else {
                    unparsed.push(part);
                }
            } else if part.len() >= 6 && &part[0..3] == "gps" {
                if let Some((first, second)) = part[3..].split_once('x') {
                    if first.parse::<u8>().is_ok() && second.parse::<u8>().is_ok() {
                        ogn_comment.gps_quality = Some(part[3..].to_string());
                    } else {
                        unparsed.push(part);
                    }
                } else {
                    unparsed.push(part);
                }
            } else if part.len() == 8 && &part[0..2] == "FL" {
                if let Ok(flight_level) = part[2..].parse::<f32>() {
                    ogn_comment.flight_level = Some(flight_level);
                } else {
                    unparsed.push(part);
                }
            } else if part.len() >= 2 && &part[0..1] == "s" {
                if let Ok(software_version) = part[1..].parse::<f32>() {
                    ogn_comment.software_version = Some(software_version);
                } else {
                    unparsed.push(part);
                }
            } else if part.len() == 3 && &part[0..1] == "h" {
                if let Ok(hardware_version) = u16::from_str_radix(&part[1..], 16) {
                    ogn_comment.hardware_version = Some(hardware_version);
                } else {
                    unparsed.push(part);
                }
            } else if part.len() == 7 && &part[0..1] == "r" {
                if let Ok(real_id) = u32::from_str_radix(&part[1..], 16) {
                    ogn_comment.real_id = Some(real_id);
                } else {
                    unparsed.push(part);
                }
            } else {
                unparsed.push(part);
            }
        }
        ogn_comment.comment = if !unparsed.is_empty() {
            Some(unparsed.join(" "))
        } else {
            None
        };
        ogn_comment
    }
}

fn split_value_unit(s: &str) -> Option<(&str, &str)> {
    let length = s.len();
    s.chars()
        .enumerate()
        .scan(
            (false, false, false),
            |(has_digits, is_signed, has_decimal), (idx, elem)| {
                if idx == 0 && ['+', '-'].contains(&elem) {
                    *is_signed = true;
                    Some((idx, *has_digits))
                } else if elem == '.' && !(*has_decimal) {
                    *has_decimal = true;
                    Some((idx, *has_digits))
                } else if elem.is_ascii_digit() {
                    *has_digits = true;
                    Some((idx, *has_digits))
                } else {
                    None
                }
            },
        )
        .last()
        .and_then(|(split_position, has_digits)| {
            if has_digits && split_position != length - 1 {
                Some((&s[..(split_position + 1)], &s[(split_position + 1)..]))
            } else {
                None
            }
        })
}

#[test]
fn test_split_value_unit() {
    assert_eq!(split_value_unit("1dB"), Some(("1", "dB")));
    assert_eq!(split_value_unit("-3kHz"), Some(("-3", "kHz")));
    assert_eq!(split_value_unit("+3.141rpm"), Some(("+3.141", "rpm")));
    assert_eq!(split_value_unit("+.1A"), Some(("+.1", "A")));
    assert_eq!(split_value_unit("-12.V"), Some(("-12.", "V")));
    assert_eq!(split_value_unit("+kVA"), None);
    assert_eq!(split_value_unit("25"), None);
}

#[test]
fn test_flr() {
    let result: OGNComment = "255/045/A=003399 !W03! id06DDFAA3 -613fpm -3.9rot 22.5dB 7e -7.0kHz gps3x7 s7.07 h41 rD002F8".into();
    assert_eq!(
        result,
        OGNComment {
            course: Some(255),
            speed: Some(45),
            altitude: Some(3399),
            additional_precision: Some(AdditionalPrecision { lat: 0, lon: 3 }),
            id: Some(ID {
                address_type: 2,
                aircraft_type: 1,
                is_stealth: false,
                is_notrack: false,
                address: 0xDDFAA3
            }),
            climb_rate: Some(-613),
            turn_rate: Some(-3.9),
            signal_quality: Some(22.5),
            error: Some(7),
            frequency_offset: Some(-7.0),
            gps_quality: Some("3x7".into()),
            software_version: Some(7.07),
            hardware_version: Some(0x41),
            real_id: Some(0xD002F8),
            ..Default::default()
        }
    );
}

#[test]
fn test_trk() {
    let result: OGNComment = "000/000/A=002280 !W59! id07395004 +000fpm +0.0rot FL021.72 40.2dB -15.1kHz gps9x13 +15.8dBm".into();
    assert_eq!(
        result,
        OGNComment {
            course: Some(0),
            speed: Some(0),
            altitude: Some(2280),
            additional_precision: Some(AdditionalPrecision { lat: 5, lon: 9 }),
            id: Some(ID {
                address_type: 3,
                aircraft_type: 1,
                is_stealth: false,
                is_notrack: false,
                address: 0x395004
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
    let result: OGNComment =
        "208/063/A=003222 !W97! id06D017DC -395fpm -2.4rot 8.2dB -6.1kHz gps2xFLRD0".into();
    assert_eq!(result.frequency_offset, Some(-6.1));
    assert_eq!(result.gps_quality.is_some(), false);
    assert_eq!(result.comment, Some("gps2xFLRD0".to_string()));
}
