extern crate actix;
extern crate actix_ogn;
extern crate pretty_env_logger;

extern crate json_patch;

use actix::*;
use actix_ogn::OGNMessage;
use aprs_parser::AprsData;
use influxdb_line_protocol::{DataPoint, FieldValue};
use json_patch::merge;
use log::{debug, error, warn};
use serde_json::json;
use std::time::SystemTime;

use crate::OutputFormat;

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
                    aprs_parser::AprsData::Position(x) => {
                        let patch = json!({
                            "messaging_supported": x.messaging_supported,
                            "latitude": *x.latitude,
                            "longitude": *x.longitude,
                            "symbol_table": x.symbol_table,
                            "symbol_code": x.symbol_code,
                            "comment": x.comment,
                        });

                        merge(&mut json_aprs, &patch);
                        //println!("{}", x.comment);
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
        match aprs_parser::parse(&message.raw) {
            Ok(value) => {
                let timestamp = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();
                let tags: Vec<(&str, &str)> = vec![
                    ("src_call", &value.from.call),
                    ("dst_call", &value.to.call),
                    ("receiver", &value.via.iter().last().unwrap().call),
                ];
                let mut fields: Vec<(&str, FieldValue)> =
                    vec![("fieldKey", FieldValue::String("fieldValue"))];
                if let AprsData::Position(pos) = value.data {
                    fields.push(("latitude", FieldValue::Float(*pos.latitude as f64)));
                    fields.push(("longitude", FieldValue::Float(*pos.longitude as f64)));
                    fields.push(("symbol_table", FieldValue::String("x")));
                    fields.push(("symbol_code", FieldValue::String("y")));
                }
                let data_point = DataPoint {
                    measurement: "myMeasurement",
                    tag_set: tags,
                    field_set: fields,
                    timestamp: Some(timestamp as i64),
                };
                print!("{}", data_point.into_string().unwrap());
            }
            Err(err) => error!("Not a valid APRS message: {}", err),
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
