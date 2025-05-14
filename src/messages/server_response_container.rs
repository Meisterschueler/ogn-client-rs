use std::time::UNIX_EPOCH;

use actix::prelude::*;
use chrono::prelude::*;
use influxdb_line_protocol::LineProtocolBuilder;
use ogn_parser::{AprsData, ServerResponse};

use crate::element_getter::ElementGetter;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ServerResponseContainer {
    pub server_response: ServerResponse,

    // every message comes from a raw string at a specific timestamp
    pub ts: DateTime<Utc>,
    pub raw_message: String,

    // if the timestamp in the message (HHMMSS or DDHHMM) differs not too much from the server timestamp (DateTime), we can cast it also to a DateTime
    pub receiver_ts: Option<DateTime<Utc>>,

    // APRS positions may have a bearing and distance to the receiver
    pub bearing: Option<f64>,
    pub distance: Option<f64>,
    pub normalized_signal_quality: Option<f64>,

    pub plausibility: Option<u16>,
}

impl ServerResponseContainer {
    pub fn csv_header_errors() -> String {
        "ts,raw_message,error_message".to_string()
    }

    pub fn csv_header_positions() -> String {
        let columns = vec![
            "ts",
            //"raw_message",
            "src_call",
            "dst_call",
            "receiver",
            "receiver_time",
            //"messaging_supported",
            //"latitude",
            //"longitude",
            "symbol_table",
            "symbol_code",
            //"comment",
            "course",
            "speed",
            "altitude",
            //"additional_lat",
            //"additional_lon",
            "address_type",
            "aircraft_type",
            "is_stealth",
            "is_notrack",
            "address",
            "climb_rate",
            "turn_rate",
            "error",
            "frequency_offset",
            "signal_quality",
            "gps_quality",
            "flight_level",
            "signal_power",
            "software_version",
            "hardware_version",
            "original_address",
            "unparsed",
            "receiver_ts",
            "bearing",
            "distance",
            "normalized_quality",
            "location",
            "plausibility",
        ];
        columns.join(",")
    }

    pub fn csv_header_statuses() -> String {
        let columns = vec![
            "ts",
            //"raw_message",
            "src_call",
            "dst_call",
            "receiver",
            "receiver_time",
            //"comment",
            "version",
            "platform",
            "cpu_load",
            "ram_free",
            "ram_total",
            "ntp_offset",
            "ntp_correction",
            "voltage",
            "amperage",
            "cpu_temperature",
            "visible_senders",
            "latency",
            "senders",
            "rf_correction_manual",
            "rf_correction_automatic",
            "noise",
            "senders_signal_quality",
            "senders_messages",
            "good_senders_signal_quality",
            "good_senders",
            "good_and_bad_senders",
            "unparsed",
            "receiver_ts",
        ];
        columns.join(",")
    }

    pub fn csv_header_server_comments() -> String {
        let columns = ["ts", "version", "server_ts", "server", "ip_address", "port"];
        columns.join(",")
    }

    pub fn to_csv(&self) -> String {
        match &self.server_response {
            ServerResponse::AprsPacket(aprs_packet) => {
                match aprs_packet.data {
                    AprsData::Position(_) => {
                        let elements = &self.get_elements();
                        format!(
                            // "\"{}\",\"{}\",{},{},{},{},{},{},{},{},{},\"{}\",{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},\"{}\",{},{},{},{},SRID=4326;POINT({} {}),{}",
                            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},SRID=4326;POINT({} {}),{}",
                            format!("\"{}\"", self.ts), // string
                            //self.raw_message.replace('"', "\"\""),   // string
                            elements.get("src_call").unwrap(),
                            elements.get("dst_call").unwrap(),
                            elements.get("receiver").unwrap(),
                            elements.get("receiver_time").unwrap_or(&"".to_string()),
                            //elements.get("messaging_supported").unwrap(),
                            //elements.get("latitude").unwrap(),
                            //elements.get("longitude").unwrap(),
                            elements.get("symbol_table").unwrap(),
                            elements.get("symbol_code").unwrap(),
                            //elements.get("comment").unwrap().replace('"', "\"\""),   // string
                            elements.get("course").unwrap_or(&"".to_string()),
                            elements.get("speed").unwrap_or(&"".to_string()),
                            elements.get("altitude").unwrap_or(&"".to_string()),
                            //elements.get("additional_lat").unwrap_or(&"".to_string()),
                            //elements.get("additional_lon").unwrap_or(&"".to_string()),
                            elements.get("address_type").unwrap_or(&"".to_string()),
                            elements.get("aircraft_type").unwrap_or(&"".to_string()),
                            elements.get("is_stealth").unwrap_or(&"".to_string()),
                            elements.get("is_notrack").unwrap_or(&"".to_string()),
                            elements.get("address").unwrap_or(&"".to_string()),
                            elements.get("climb_rate").unwrap_or(&"".to_string()),
                            elements.get("turn_rate").unwrap_or(&"".to_string()),
                            elements.get("error").unwrap_or(&"".to_string()),
                            elements.get("frequency_offset").unwrap_or(&"".to_string()),
                            elements.get("signal_quality").unwrap_or(&"".to_string()),
                            elements.get("gps_quality").unwrap_or(&"".to_string()),
                            elements.get("flight_level").unwrap_or(&"".to_string()),
                            elements.get("signal_power").unwrap_or(&"".to_string()),
                            elements.get("software_version").unwrap_or(&"".to_string()),
                            elements.get("hardware_version").unwrap_or(&"".to_string()),
                            elements.get("original_address").unwrap_or(&"".to_string()),
                            format!(
                                "\"{}\"",
                                elements
                                    .get("unparsed")
                                    .unwrap_or(&"".to_string())
                                    .replace('"', "\"\"")
                            ), // string
                            elements
                                .get("receiver_ts")
                                .map(|s| format!("\"{s}\""))
                                .unwrap_or("".to_string()), // string
                            elements.get("bearing").unwrap_or(&"".to_string()),
                            elements.get("distance").unwrap_or(&"".to_string()),
                            elements
                                .get("normalized_signal_quality")
                                .unwrap_or(&"".to_string()),
                            elements.get("longitude").unwrap(),
                            elements.get("latitude").unwrap(),
                            elements.get("plausibility").unwrap_or(&"".to_string()),
                        )
                    }
                    AprsData::Status(_) => {
                        let elements = &self.get_elements();

                        format!(
                            // "\"{}\",\"{}\",{},{},{},{},\"{}\",{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},\"{}\",{}",
                            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                            format!("\"{}\"", self.ts), // string
                            //self.raw_message.replace('"', "\"\""), // string
                            elements.get("src_call").unwrap(),
                            elements.get("dst_call").unwrap(),
                            elements.get("receiver").unwrap(),
                            elements.get("receiver_time").unwrap_or(&"".to_string()),
                            //elements.get("comment").unwrap().replace('"', "\"\""), // string
                            elements.get("version").unwrap_or(&"".to_string()),
                            elements.get("platform").unwrap_or(&"".to_string()),
                            elements.get("cpu_load").unwrap_or(&"".to_string()),
                            elements.get("ram_free").unwrap_or(&"".to_string()),
                            elements.get("ram_total").unwrap_or(&"".to_string()),
                            elements.get("ntp_offset").unwrap_or(&"".to_string()),
                            elements.get("ntp_correction").unwrap_or(&"".to_string()),
                            elements.get("voltage").unwrap_or(&"".to_string()),
                            elements.get("amperage").unwrap_or(&"".to_string()),
                            elements.get("cpu_temperature").unwrap_or(&"".to_string()),
                            elements.get("visible_senders").unwrap_or(&"".to_string()),
                            elements.get("latency").unwrap_or(&"".to_string()),
                            elements.get("senders").unwrap_or(&"".to_string()),
                            elements
                                .get("rf_correction_manual")
                                .unwrap_or(&"".to_string()),
                            elements
                                .get("rf_correction_automatic")
                                .unwrap_or(&"".to_string()),
                            elements.get("noise").unwrap_or(&"".to_string()),
                            elements
                                .get("senders_signal_quality")
                                .unwrap_or(&"".to_string()),
                            elements.get("senders_messages").unwrap_or(&"".to_string()),
                            elements
                                .get("good_senders_signal_quality")
                                .unwrap_or(&"".to_string()),
                            elements.get("good_senders").unwrap_or(&"".to_string()),
                            elements
                                .get("good_and_bad_senders")
                                .unwrap_or(&"".to_string()),
                            format!(
                                "\"{}\"",
                                elements
                                    .get("unparsed")
                                    .unwrap_or(&"".to_string())
                                    .replace('"', "\"\"")
                            ), // string
                            elements
                                .get("receiver_ts")
                                .map(|s| format!("\"{s}\""))
                                .unwrap_or("".to_string()), // string,
                        )
                    }
                    AprsData::Message(_) => {
                        todo!()
                    }
                    AprsData::Unknown => todo!(),
                }
            }
            ServerResponse::ServerComment(server_comment) => format!(
                "\"{}\",{},\"{}\",\"{}\",\"{}\",{}",
                self.ts.to_rfc3339_opts(SecondsFormat::Nanos, true),
                server_comment.version,
                server_comment
                    .timestamp
                    .to_rfc3339_opts(SecondsFormat::Nanos, true),
                server_comment.server,
                server_comment.ip_address,
                server_comment.port
            ),
            ServerResponse::ParserError(parser_error) => format!(
                "\"{}\",\"{}\",\"{}\"",
                self.ts.to_rfc3339_opts(SecondsFormat::Nanos, true),
                self.raw_message.replace('"', "\"\""),
                parser_error.to_string().replace('"', "\"\""),
            ),
            ServerResponse::Comment(_) => format!("\"{}\"", self.raw_message.replace('"', "\"\"")),
        }
    }

    fn get_tags(&self) -> Vec<(&str, String)> {
        let elements = self.get_elements();
        elements
            .iter()
            .filter(|&(k, _)| ["src_call", "dst_call", "receiver"].contains(k))
            .map(|(k, v)| (*k, v.clone()))
            .collect()
    }

    fn get_fields(&self) -> Vec<(&str, String)> {
        let elements = self.get_elements();
        elements
            .iter()
            .filter(|&(k, _)| !["src_call", "dst_call", "receiver"].contains(k))
            .map(|(k, v)| (*k, v.to_string()))
            .collect()
    }

    pub fn to_ilp(&self) -> String {
        let measurement = match &self.server_response {
            ServerResponse::AprsPacket(aprs_packet) => match aprs_packet.data {
                AprsData::Position(_) => "positions",
                AprsData::Status(_) => "statuses",
                AprsData::Message(_) => "messages",
                AprsData::Unknown => "unknowns",
            },
            ServerResponse::ServerComment(_) => "server_comments",
            ServerResponse::ParserError(_) => "errors",
            ServerResponse::Comment(_) => "comments",
        };

        let mut lp = LineProtocolBuilder::new().measurement(measurement);
        for (key, value) in self.get_tags().into_iter() {
            lp = lp.tag(key, value.as_str());
        }
        let mut field_iter = self.get_fields().into_iter();
        let (key, value) = field_iter.next().unwrap();
        let mut lp = lp.field(key, value.as_str());
        for (key, value) in field_iter {
            lp = lp.field(key, value.as_str());
        }
        let lp = lp
            .timestamp(
                self.ts
                    .signed_duration_since(DateTime::<Utc>::from(UNIX_EPOCH))
                    .num_nanoseconds()
                    .unwrap(),
            )
            .close_line();
        String::from_utf8_lossy(&lp.build()).into_owned()
    }
}
