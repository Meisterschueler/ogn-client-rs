extern crate actix;
extern crate actix_ogn;
extern crate pretty_env_logger;

extern crate json_patch;

use actix::*;
use actix_ogn::OGNMessage;
use aprs_parser::{AprsData, AprsPosition};
use influxdb_line_protocol::{DataPoint, FieldValue};
use json_patch::merge;
use log::{debug, error, warn};
use serde_json::json;
use std::time::SystemTime;

use crate::OutputFormat;
use crate::OGNComment;

pub struct ConsoleLogger {
    pub format: OutputFormat,
    pub includes: Option<Vec<String>>,
    pub excludes: Option<Vec<String>>,
}

impl Actor for ConsoleLogger {
    type Context = Context<Self>;
}

impl ConsoleLogger {
    fn print_raw(&mut self, message: &OGNMessage) {
        let ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        println!("{ts}: {aprs}", aprs = message.raw);
    }

    fn print_json(&mut self, message: &OGNMessage) {
        match aprs_parser::parse(&message.raw) {
            Ok(value) => {
                let mut json_aprs = json!({
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
                        let comment: &str = &ogn_comment.comment.unwrap_or_default();
                        if comment != "" {
                            merge(&mut json_aprs, &json!({"comment": comment}));
                        }
                    }
                    aprs_parser::AprsData::Message(_) => {}
                    _ => {}
                };
                println!("{}", json_aprs.to_string());
            }
            Err(err) => error!("Not a valid APRS message: {}", err),
        }
    }

    fn print_influxdb(&mut self, message: &OGNMessage) {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        match aprs_parser::parse(&message.raw) {
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
                    let comment: &str = &ogn_comment.comment.unwrap_or_default();
                    if comment != "" {
                        fields.push(("comment", FieldValue::String(comment)));
                    }
                    
                    let data_point = DataPoint {
                        measurement: "ogn_position",
                        tag_set: tags,
                        field_set: fields,
                        timestamp: Some(timestamp as i64),
                    };
                    let data = data_point.into_string().unwrap();
                    print!("{data}");
                } else {
                    let data_point = DataPoint {
                        measurement: "ogn_unparsed",
                        tag_set: tags,
                        field_set: vec![("message", FieldValue::String(&message.raw))],
                        timestamp: Some(timestamp as i64),
                    };
                    let data = data_point.into_string().unwrap();
                    print!("{data}");
                }
            }
            Err(err) => {
                let error_string = err.to_string();
                let data_point = DataPoint {
                    measurement: "ogn_error",
                    tag_set: vec![],
                    field_set: vec![("error", FieldValue::String(&error_string)), ("message", FieldValue::String(&message.raw))],
                    timestamp: Some(timestamp as i64),
                };
                let data = data_point.into_string().unwrap();
                print!("{data}");
            }
        }
    }
}

impl Handler<OGNMessage> for ConsoleLogger {
    type Result = ();
    fn handle(&mut self, message: OGNMessage, _: &mut Context<Self>) {
        let passes_filter = if let Some(includes) = &self.includes {
            includes
                .iter()
                .any(|substring| message.raw.contains(substring))
        } else if let Some(excludes) = &self.excludes {
            !excludes
                .iter()
                .any(|substring| message.raw.contains(substring))
        } else {
            true
        };

        if passes_filter {
            match self.format {
                OutputFormat::Raw => self.print_raw(&message),
                OutputFormat::Json => self.print_json(&message),
                OutputFormat::Influx => self.print_influxdb(&message),
            }
        }
    }
}
