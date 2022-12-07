use json_patch::merge;
use serde_json::json;
use actix_ogn::OGNMessage;
use aprs_parser::AprsData;
use influxdb_line_protocol::{DataPoint, FieldValue};
use log::error;

use crate::OGNComment;


pub trait OGNMessageConverter {
    fn to_raw(&self, ts: u128) -> String;
    fn to_json(&self, ts: u128) -> String;
    fn to_influx(&self, ts: u128) -> String;
}

impl OGNMessageConverter for OGNMessage {
    fn to_raw(&self, ts: u128) -> String {
        format!("{ts}: {aprs}", aprs = self.raw)
    }

    fn to_json(&self, ts: u128) -> String {
        match aprs_parser::parse(&self.raw) {
            Ok(value) => {
                let mut json_aprs = json!({
                    "ts": ts,
                    "src_call": value.from.call,
                    "dst_call": value.to.call,
                    "receiver": value.via.iter().last().cloned().unwrap().call,
                });
                match value.data {
                    aprs_parser::AprsData::Position(pos) => {
                        let ogn_comment: OGNComment = pos.comment.as_str().into();
                        let mut latitude: f64 = *pos.latitude as f64;
                        let mut longitude: f64 = *pos.longitude as f64;
                        if let Some(additional_precision) = ogn_comment.additional_precision {
                            latitude += (additional_precision.lat as f64) / 1000.0;
                            longitude += (additional_precision.lon as f64) / 1000.0;
                        }

                        let patch = json!({
                            "latitude": latitude,
                            "longitude": longitude,
                            "symbol_table": pos.symbol_table,
                            "symbol_code": pos.symbol_code,
                        });

                        merge(&mut json_aprs, &patch);
                        
                        if let Some(course) = ogn_comment.course {
                            merge(&mut json_aprs, &json!({"course": course}));
                        }
                        if let Some(speed) = ogn_comment.speed {
                            merge(&mut json_aprs, &json!({"speed": speed}));
                        }
                        if let Some(altitude) = ogn_comment.altitude {
                            merge(&mut json_aprs, &json!({"altitude": altitude}));
                        }
                        if let Some(id) = ogn_comment.id {
                            merge(&mut json_aprs, &json!({"address_type": id.address_type}));
                            merge(&mut json_aprs, &json!({"aircraft_type": id.aircraft_type}));
                            merge(&mut json_aprs, &json!({"is_stealth": id.is_stealth}));
                            merge(&mut json_aprs, &json!({"is_notrack": id.is_notrack}));
                            merge(&mut json_aprs, &json!({"address": id.address}));
                        }
                        if let Some(climb_rate) = ogn_comment.climb_rate {
                            merge(&mut json_aprs, &json!({"climb_rate": climb_rate}));
                        }
                        if let Some(turn_rate) = ogn_comment.turn_rate {
                            merge(&mut json_aprs, &json!({"turn_rate": turn_rate}));
                        }
                        if let Some(error) = ogn_comment.error {
                            merge(&mut json_aprs, &json!({"error": error}));
                        }
                        if let Some(frequency_offset) = ogn_comment.frequency_offset {
                            merge(&mut json_aprs, &json!({"frequency_offset": frequency_offset}));
                        }
                        if let Some(signal_quality) = ogn_comment.signal_quality {
                            merge(&mut json_aprs, &json!({"signal_quality": signal_quality}));
                        }
                        if let Some(gps_quality) = ogn_comment.gps_quality {
                            merge(&mut json_aprs, &json!({"gps_quality": gps_quality}));
                        }
                        if let Some(flight_level) = ogn_comment.flight_level {
                            merge(&mut json_aprs, &json!({"flight_level" :flight_level}));
                        }
                        if let Some(signal_power) = ogn_comment.signal_power {
                            merge(&mut json_aprs, &json!({"signal_power": signal_power}));
                        }
                        if let Some(software_version) = ogn_comment.software_version {
                            merge(&mut json_aprs, &json!({"software_version": software_version}));
                        }
                        if let Some(hardware_version) = ogn_comment.hardware_version {
                            merge(&mut json_aprs, &json!({"hardware_version": hardware_version}));
                        }
                        if let Some(real_id) = ogn_comment.real_id {
                            merge(&mut json_aprs, &json!({"real_id": real_id}));
                        }

                        let comment: &str = &ogn_comment.comment.unwrap_or_default();
                        if comment != "" {
                            merge(&mut json_aprs, &json!({"comment": comment}));
                        }
                    }
                    aprs_parser::AprsData::Message(_) => {}
                    _ => {}
                };
                json_aprs.to_string()
            }
            Err(err) => {
                error!("Not a valid APRS message: {}", err);
                String::new()
            }
        }
    }

    fn to_influx(&self, ts: u128) -> String {
        match aprs_parser::parse(&self.raw) {
            Ok(value) => {
                let tags: Vec<(&str, &str)> = vec![
                    ("src_call", &value.from.call),
                    ("dst_call", &value.to.call),
                    ("receiver", &value.via.iter().last().unwrap().call),
                ];

                if let AprsData::Position(pos) = value.data {
                    let mut fields: Vec<(&str, FieldValue)> = vec![];
                    
                    let ogn_comment: OGNComment = pos.comment.as_str().into();
                    let symbol_table: &str = &pos.symbol_table.to_string();
                    let symbol_code: &str = &pos.symbol_code.to_string();
                    let mut latitude: f64 = *pos.latitude as f64;
                    let mut longitude: f64 = *pos.longitude as f64;
                    if let Some(additional_precision) = ogn_comment.additional_precision {
                        latitude += (additional_precision.lat as f64) / 1000.0;
                        longitude += (additional_precision.lon as f64) / 1000.0;
                    }

                    fields.push(("latitude", FieldValue::Float(latitude)));
                    fields.push(("longitude", FieldValue::Float(longitude)));
                    fields.push(("symbol_table", FieldValue::String(symbol_table)));
                    fields.push(("symbol_code", FieldValue::String(symbol_code)));

                    if let Some(course) = ogn_comment.course {
                        fields.push(("course", FieldValue::Float(course.into())));
                    }
                    if let Some(speed) = ogn_comment.speed {
                        fields.push(("speed", FieldValue::Float(speed.into())));
                    }
                    if let Some(altitude) = ogn_comment.altitude {
                        fields.push(("altitude", FieldValue::Float(altitude.into())));
                    }
                    if let Some(id) = ogn_comment.id {
                        fields.push(("address_type", FieldValue::Integer(id.address_type.into())));
                        fields.push(("aircraft_type", FieldValue::Integer(id.aircraft_type.into())));
                        fields.push(("is_stealth", FieldValue::Boolean(id.is_stealth)));
                        fields.push(("is_notrack", FieldValue::Boolean(id.is_notrack)));
                        fields.push(("address", FieldValue::Integer(id.address.into())));
                    }
                    if let Some(climb_rate) = ogn_comment.climb_rate {
                        fields.push(("climb_rate", FieldValue::Integer(climb_rate as i64)));
                    }
                    if let Some(turn_rate) = ogn_comment.turn_rate {
                        fields.push(("turn_rate", FieldValue::Float(turn_rate as f64)));
                    }
                    if let Some(error) = ogn_comment.error {
                        fields.push(("error", FieldValue::Integer(error as i64)));
                    }
                    if let Some(frequency_offset) = ogn_comment.frequency_offset {
                        fields.push(("frequency_offset", FieldValue::Float(frequency_offset as f64)));
                    }
                    if let Some(signal_quality) = ogn_comment.signal_quality {
                        fields.push(("signal_quality", FieldValue::Float(signal_quality as f64)));
                    }
                    let gps_quality: &str = &ogn_comment.gps_quality.unwrap_or_default();
                    if gps_quality != "" {
                        fields.push(("gps_quality", FieldValue::String(gps_quality)));
                    }
                    if let Some(flight_level) = ogn_comment.flight_level {
                        fields.push(("flight_level", FieldValue::Float(flight_level as f64)));
                    }
                    if let Some(signal_power) = ogn_comment.signal_power {
                        fields.push(("signal_power", FieldValue::Float(signal_power as f64)));
                    }
                    if let Some(software_version) = ogn_comment.software_version {
                        fields.push(("software_version", FieldValue::Float(software_version as f64)));
                    }
                    if let Some(hardware_version) = ogn_comment.hardware_version {
                        fields.push(("hardware_version", FieldValue::Integer(hardware_version as i64)));
                    }
                    if let Some(real_id) = ogn_comment.real_id {
                        fields.push(("real_id", FieldValue::Integer(real_id as i64)));
                    }
                    let comment: &str = &ogn_comment.comment.unwrap_or_default();
                    if comment != "" {
                        fields.push(("comment", FieldValue::String(comment)));
                    }
                    
                    let data_point = DataPoint {
                        measurement: "ogn_position",
                        tag_set: tags,
                        field_set: fields,
                        timestamp: Some(ts as i64),
                    };
                    data_point.into_string().unwrap()
                } else {
                    let data_point = DataPoint {
                        measurement: "ogn_unparsed",
                        tag_set: tags,
                        field_set: vec![("message", FieldValue::String(&self.raw))],
                        timestamp: Some(ts as i64),
                    };
                    data_point.into_string().unwrap()
                }
            }
            Err(err) => {
                let error_string = err.to_string();
                let data_point = DataPoint {
                    measurement: "ogn_error",
                    tag_set: vec![],
                    field_set: vec![("error", FieldValue::String(&error_string)), ("message", FieldValue::String(&self.raw))],
                    timestamp: Some(ts as i64),
                };
                data_point.into_string().unwrap()
            }
        }
    }
}