use std::{collections::HashMap, time::UNIX_EPOCH};

use aprs_parser::{AprsData, AprsPacket, AprsPosition, AprsStatus};
use chrono::{DateTime, SecondsFormat, Utc};
use influxdb_line_protocol::LineProtocolBuilder;

use crate::{
    date_time_guesser::DateTimeGuesser, distance_service::Relation, PositionComment, StatusComment,
};

pub trait ElementGetter {
    fn get_elements(&self) -> HashMap<&str, String>;
}

pub trait CsvSerializer {
    fn csv_header() -> String;
    fn to_csv(&self) -> String;
    fn get_tags(&self) -> Vec<(&str, String)>;
    fn get_fields(&self) -> Vec<(&str, String)>;
    fn to_ilp(
        measurement: &str,
        tags: Vec<(&str, String)>,
        fields: Vec<(&str, String)>,
        ts: DateTime<Utc>,
    ) -> String {
        let mut lp = LineProtocolBuilder::new().measurement(measurement);
        for (key, value) in tags {
            lp = lp.tag(key, value.as_str());
        }
        let mut field_iter = fields.into_iter();
        let (key, value) = field_iter.next().unwrap();
        let mut lp = lp.field(key, value.as_str());
        for (key, value) in field_iter {
            lp = lp.field(key, value.as_str());
        }
        let lp = lp
            .timestamp(
                ts.signed_duration_since(DateTime::<Utc>::from(UNIX_EPOCH))
                    .num_nanoseconds()
                    .unwrap(),
            )
            .close_line();
        String::from_utf8_lossy(&lp.build()).into_owned()
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct OGNPacketInvalid {
    pub ts: DateTime<Utc>,
    pub raw_message: String,
    pub error_message: String,
}

impl CsvSerializer for OGNPacketInvalid {
    fn csv_header() -> String {
        "ts,raw_message,error_message".to_string()
    }

    fn to_csv(&self) -> String {
        format!(
            "\"{}\",\"{}\",\"{}\"",
            self.ts.to_rfc3339_opts(SecondsFormat::Nanos, true),
            self.raw_message.replace('"', "\"\""),
            self.error_message.replace('"', "\"\""),
        )
    }

    fn get_tags(&self) -> Vec<(&str, String)> {
        vec![]
    }

    fn get_fields(&self) -> Vec<(&str, String)> {
        [
            ("raw_message", self.raw_message.clone()),
            ("error_message", self.error_message.clone()),
        ]
        .to_vec()
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct OGNPacketUnknown {
    pub ts: DateTime<Utc>,
    pub raw_message: String,
    pub src_call: String,
    pub dst_call: String,
    pub receiver: String,
}

impl CsvSerializer for OGNPacketUnknown {
    fn csv_header() -> String {
        "ts,raw_message,src_call,dst_call,receiver".to_string()
    }

    fn to_csv(&self) -> String {
        format!(
            "\"{}\",\"{}\",{},{},{}",
            self.ts.to_rfc3339_opts(SecondsFormat::Nanos, true),
            self.raw_message.replace('"', "\"\""),
            self.src_call,
            self.dst_call,
            self.receiver
        )
    }

    fn get_tags(&self) -> Vec<(&str, String)> {
        [
            ("src_call", self.src_call.clone()),
            ("dst_call", self.dst_call.clone()),
            ("receiver", self.receiver.clone()),
        ]
        .to_vec()
    }

    fn get_fields(&self) -> Vec<(&str, String)> {
        [("raw_message", self.raw_message.clone())].to_vec()
    }
}

#[derive(Debug, PartialEq)]
pub struct OGNPacketPosition {
    pub ts: DateTime<Utc>,
    pub raw_message: String,
    pub src_call: String,
    pub dst_call: String,
    pub receiver: String,
    pub aprs: AprsPosition,
    pub comment: PositionComment,

    pub receiver_ts: Option<DateTime<Utc>>,
    pub relation: Option<Relation>,
    pub normalized_quality: Option<f64>,
}

impl CsvSerializer for OGNPacketPosition {
    fn csv_header() -> String {
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
        ];
        columns.join(",")
    }

    fn to_csv(&self) -> String {
        let head = self.get_elements();
        let body = self.comment.get_elements();

        format!(
            // "\"{}\",\"{}\",{},{},{},{},{},{},{},{},{},\"{}\",{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},\"{}\",{},{},{},{},SRID=4326;POINT({} {})",
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},SRID=4326;POINT({} {})",
            format!("\"{}\"", head.get("ts").unwrap()),    // string
            //head.get("raw_message").unwrap().replace('"', "\"\""),   // string
            head.get("src_call").unwrap(),
            head.get("dst_call").unwrap(),
            head.get("receiver").unwrap(),
            head.get("receiver_time").unwrap(),
            //head.get("messaging_supported").unwrap(),
            //head.get("latitude").unwrap(),
            //head.get("longitude").unwrap(),
            head.get("symbol_table").unwrap(),
            head.get("symbol_code").unwrap(),
            //head.get("comment").unwrap().replace('"', "\"\""),   // string
            body.get("course").unwrap_or(&"".to_string()),
            body.get("speed").unwrap_or(&"".to_string()),
            body.get("altitude").unwrap_or(&"".to_string()),
            //body.get("additional_lat").unwrap_or(&"".to_string()),
            //body.get("additional_lon").unwrap_or(&"".to_string()),
            body.get("address_type").unwrap_or(&"".to_string()),
            body.get("aircraft_type").unwrap_or(&"".to_string()),
            body.get("is_stealth").unwrap_or(&"".to_string()),
            body.get("is_notrack").unwrap_or(&"".to_string()),
            body.get("address").unwrap_or(&"".to_string()),
            body.get("climb_rate").unwrap_or(&"".to_string()),
            body.get("turn_rate").unwrap_or(&"".to_string()),
            body.get("error").unwrap_or(&"".to_string()),
            body.get("frequency_offset").unwrap_or(&"".to_string()),
            body.get("signal_quality").unwrap_or(&"".to_string()),
            body.get("gps_quality").unwrap_or(&"".to_string()),
            body.get("flight_level").unwrap_or(&"".to_string()),
            body.get("signal_power").unwrap_or(&"".to_string()),
            body.get("software_version").unwrap_or(&"".to_string()),
            body.get("hardware_version").unwrap_or(&"".to_string()),
            body.get("original_address").unwrap_or(&"".to_string()),
            format!("\"{}\"", body.get("unparsed").unwrap_or(&"".to_string()).replace('"', "\"\"")),    // string
            head.get("receiver_ts").map(|s| format!("\"{s}\"")).unwrap_or("".to_string()),    // string
            head.get("bearing").unwrap_or(&"".to_string()),
            head.get("distance").unwrap_or(&"".to_string()),
            head.get("normalized_quality").unwrap_or(&"".to_string()),
            head.get("longitude").unwrap(),
            head.get("latitude").unwrap(),
        )
    }

    fn get_tags(&self) -> Vec<(&str, String)> {
        let head = self.get_elements();
        //let body = self.comment.get_elements();

        let mut tags = vec![];
        tags.push(("src_call", head.get("src_call").unwrap().to_string()));
        tags.push(("dst_call", head.get("dst_call").unwrap().to_string()));
        tags.push(("receiver", head.get("receiver").unwrap().to_string()));
        tags.push((
            "messaging_supported",
            head.get("messaging_supported").unwrap().to_string(),
        ));
        tags.push((
            "symbol_table",
            head.get("symbol_table").unwrap().to_string(),
        ));
        tags.push(("symbol_code", head.get("symbol_code").unwrap().to_string()));

        tags
    }

    fn get_fields(&self) -> Vec<(&str, String)> {
        let head = self.get_elements();
        let body = self.comment.get_elements();

        let mut fields = vec![];
        fields.push(("raw_message", head.get("raw_message").unwrap().to_string()));
        if let Some(s) = head.get("receiver_time") {
            fields.push(("receiver_time", s.to_string()));
        };
        if let Some(s) = head.get("latitude") {
            fields.push(("latitude", s.to_string()));
        };
        if let Some(s) = head.get("longitude") {
            fields.push(("longitude", s.to_string()));
        };
        if let Some(s) = head.get("comment") {
            fields.push(("comment", s.to_string()));
        };
        if let Some(s) = body.get("course") {
            fields.push(("course", s.to_string()));
        };
        if let Some(s) = body.get("speed") {
            fields.push(("speed", s.to_string()));
        };
        if let Some(s) = body.get("altitude") {
            fields.push(("altitude", s.to_string()));
        };
        if let Some(s) = body.get("additional_lat") {
            fields.push(("additional_lat", s.to_string()));
        };
        if let Some(s) = body.get("additional_lon") {
            fields.push(("additional_lon", s.to_string()));
        };
        if let Some(s) = body.get("address_type") {
            fields.push(("address_type", s.to_string()));
        };
        if let Some(s) = body.get("aircraft_type") {
            fields.push(("aircraft_type", s.to_string()));
        };
        if let Some(s) = body.get("is_stealth") {
            fields.push(("is_stealth", s.to_string()));
        };
        if let Some(s) = body.get("is_notrack") {
            fields.push(("is_notrack", s.to_string()));
        };
        if let Some(s) = body.get("address") {
            fields.push(("address", s.to_string()));
        };
        if let Some(s) = body.get("climb_rate") {
            fields.push(("climb_rate", s.to_string()));
        };
        if let Some(s) = body.get("turn_rate") {
            fields.push(("turn_rate", s.to_string()));
        };
        if let Some(s) = body.get("error") {
            fields.push(("error", s.to_string()));
        };
        if let Some(s) = body.get("frequency_offset") {
            fields.push(("frequency_offset", s.to_string()));
        };
        if let Some(s) = body.get("signal_quality") {
            fields.push(("signal_quality", s.to_string()));
        };
        if let Some(s) = body.get("gps_quality") {
            fields.push(("gps_quality", s.to_string()));
        };
        if let Some(s) = body.get("flight_level") {
            fields.push(("flight_level", s.to_string()));
        };
        if let Some(s) = body.get("signal_power") {
            fields.push(("signal_power", s.to_string()));
        };
        if let Some(s) = body.get("software_version") {
            fields.push(("software_version", s.to_string()));
        };
        if let Some(s) = body.get("hardware_version") {
            fields.push(("hardware_version", s.to_string()));
        };
        if let Some(s) = body.get("original_address") {
            fields.push(("original_address", s.to_string()));
        };
        if let Some(s) = body.get("unparsed") {
            fields.push(("unparsed", s.to_string()));
        };
        if let Some(s) = head.get("receiver_ts") {
            fields.push(("receiver_ts", s.to_string()));
        };
        if let Some(s) = head.get("bearing") {
            fields.push(("bearing", s.to_string()));
        };
        if let Some(s) = head.get("distance") {
            fields.push(("distance", s.to_string()));
        };
        if let Some(s) = head.get("normalized_quality") {
            fields.push(("normalized_quality", s.to_string()));
        };

        fields
    }
}

impl ElementGetter for OGNPacketPosition {
    fn get_elements(&self) -> HashMap<&str, String> {
        let mut elements: HashMap<&str, String> = HashMap::new();
        elements.insert("ts", self.ts.to_rfc3339_opts(SecondsFormat::Nanos, true));
        elements.insert("raw_message", self.raw_message.to_string());
        elements.insert("src_call", self.src_call.to_string());
        elements.insert("dst_call", self.dst_call.to_string());
        elements.insert("receiver", self.receiver.to_string());
        elements.insert(
            "messaging_supported",
            self.aprs.messaging_supported.to_string(),
        );
        elements.insert("symbol_table", self.aprs.symbol_table.to_string());
        elements.insert("symbol_code", self.aprs.symbol_code.to_string());
        if let Some(receiver_time) = &self.aprs.timestamp {
            elements.insert("receiver_time", receiver_time.to_string());
        };
        elements.insert("latitude", self.aprs.latitude.to_string());
        elements.insert("longitude", self.aprs.longitude.to_string());
        elements.insert("comment", self.aprs.comment.to_string());
        elements.extend(self.comment.get_elements());
        if let Some(receiver_ts) = self.receiver_ts {
            elements.insert("receiver_ts", receiver_ts.to_string());
        };
        if let Some(relation) = self.relation {
            elements.insert("bearing", relation.bearing.to_string());
            elements.insert("distance", relation.distance.to_string());
        }
        if let Some(normalized_quality) = self.normalized_quality {
            elements.insert("normalized_quality", normalized_quality.to_string());
        };

        elements
    }
}

#[derive(Debug, PartialEq)]
pub struct OGNPacketStatus {
    pub ts: DateTime<Utc>,
    pub raw_message: String,
    pub src_call: String,
    pub dst_call: String,
    pub receiver: String,
    pub aprs: AprsStatus,
    pub comment: StatusComment,

    pub receiver_ts: Option<DateTime<Utc>>,
}

impl CsvSerializer for OGNPacketStatus {
    fn csv_header() -> String {
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

    fn to_csv(&self) -> String {
        let head = self.get_elements();
        let body = self.comment.get_elements();

        format!(
            // "\"{}\",\"{}\",{},{},{},{},\"{}\",{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},\"{}\",{}",
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            format!("\"{}\"", head.get("ts").unwrap()),          // string
            //head.get("raw_message").unwrap().replace('"', "\"\""), // string
            head.get("src_call").unwrap(),
            head.get("dst_call").unwrap(),
            head.get("receiver").unwrap(),
            head.get("receiver_time").unwrap(),
            //head.get("comment").unwrap().replace('"', "\"\""), // string
            body.get("version").unwrap_or(&"".to_string()),
            body.get("platform").unwrap_or(&"".to_string()),
            body.get("cpu_load").unwrap_or(&"".to_string()),
            body.get("ram_free").unwrap_or(&"".to_string()),
            body.get("ram_total").unwrap_or(&"".to_string()),
            body.get("ntp_offset").unwrap_or(&"".to_string()),
            body.get("ntp_correction").unwrap_or(&"".to_string()),
            body.get("voltage").unwrap_or(&"".to_string()),
            body.get("amperage").unwrap_or(&"".to_string()),
            body.get("cpu_temperature").unwrap_or(&"".to_string()),
            body.get("visible_senders").unwrap_or(&"".to_string()),
            body.get("latency").unwrap_or(&"".to_string()),
            body.get("senders").unwrap_or(&"".to_string()),
            body.get("rf_correction_manual").unwrap_or(&"".to_string()),
            body.get("rf_correction_automatic").unwrap_or(&"".to_string()),
            body.get("noise").unwrap_or(&"".to_string()),
            body.get("senders_signal_quality").unwrap_or(&"".to_string()),
            body.get("senders_messages").unwrap_or(&"".to_string()),
            body.get("good_senders_signal_quality").unwrap_or(&"".to_string()),
            body.get("good_senders").unwrap_or(&"".to_string()),
            body.get("good_and_bad_senders").unwrap_or(&"".to_string()),
            format!("\"{}\"", body.get("unparsed").unwrap_or(&"".to_string()).replace('"', "\"\"")),   // string
            head.get("receiver_ts").map(|s| format!("\"{s}\"")).unwrap_or("".to_string()),    // string,
        )
    }

    fn get_tags(&self) -> Vec<(&str, String)> {
        let head = self.get_elements();
        //let body = self.comment.get_elements();

        let mut tags = vec![];
        tags.push(("src_call", head.get("src_call").unwrap().clone()));
        tags.push(("dst_call", head.get("dst_call").unwrap().clone()));
        tags.push(("receiver", head.get("receiver").unwrap().clone()));

        tags
    }

    fn get_fields(&self) -> Vec<(&str, String)> {
        let head = self.get_elements();
        let body = self.comment.get_elements();

        let mut fields = vec![];
        fields.push(("raw_message", head.get("raw_message").unwrap().clone()));
        fields.push(("receiver_time", head.get("receiver_time").unwrap().clone()));
        fields.push(("comment", head.get("comment").unwrap().clone()));

        if let Some(s) = body.get("version") {
            fields.push(("version", s.to_string()));
        }
        if let Some(s) = body.get("platform") {
            fields.push(("platform", s.to_string()));
        }
        if let Some(s) = body.get("cpu_load") {
            fields.push(("cpu_load", s.to_string()));
        }
        if let Some(s) = body.get("ram_free") {
            fields.push(("ram_free", s.to_string()));
        }
        if let Some(s) = body.get("ram_total") {
            fields.push(("ram_total", s.to_string()));
        }
        if let Some(s) = body.get("ntp_offset") {
            fields.push(("ntp_offset", s.to_string()));
        }
        if let Some(s) = body.get("ntp_correction") {
            fields.push(("ntp_correction", s.to_string()));
        }
        if let Some(s) = body.get("voltage") {
            fields.push(("voltage", s.to_string()));
        }
        if let Some(s) = body.get("amperage") {
            fields.push(("amperage", s.to_string()));
        }
        if let Some(s) = body.get("cpu_temperature") {
            fields.push(("cpu_temperature", s.to_string()));
        }
        if let Some(s) = body.get("visible_senders") {
            fields.push(("visible_senders", s.to_string()));
        }
        if let Some(s) = body.get("latency") {
            fields.push(("latency", s.to_string()));
        }
        if let Some(s) = body.get("senders") {
            fields.push(("senders", s.to_string()));
        }
        if let Some(s) = body.get("rf_correction_manual") {
            fields.push(("rf_correction_manual", s.to_string()));
        }
        if let Some(s) = body.get("rf_correction_automatic") {
            fields.push(("rf_correction_automatic", s.to_string()));
        }
        if let Some(s) = body.get("noise") {
            fields.push(("noise", s.to_string()));
        }
        if let Some(s) = body.get("senders_signal_quality") {
            fields.push(("senders_signal_quality", s.to_string()));
        }
        if let Some(s) = body.get("senders_messages") {
            fields.push(("senders_messages", s.to_string()));
        }
        if let Some(s) = body.get("good_senders_signal_quality") {
            fields.push(("good_senders_signal_quality", s.to_string()));
        }
        if let Some(s) = body.get("good_senders") {
            fields.push(("good_senders", s.to_string()));
        }
        if let Some(s) = body.get("good_and_bad_senders") {
            fields.push(("good_and_bad_senders", s.to_string()));
        }
        if let Some(s) = body.get("unparsed") {
            fields.push(("unparsed", s.to_string()));
        }
        if let Some(s) = body.get("receiver_ts") {
            fields.push(("receiver_ts", s.to_string()));
        }
        fields
    }
}

impl ElementGetter for OGNPacketStatus {
    fn get_elements(&self) -> HashMap<&str, String> {
        let mut elements: HashMap<&str, String> = HashMap::new();
        elements.insert("ts", self.ts.to_rfc3339_opts(SecondsFormat::Nanos, true));
        elements.insert("raw_message", self.raw_message.to_string());
        elements.insert("src_call", self.src_call.to_string());
        elements.insert("dst_call", self.dst_call.to_string());
        elements.insert("receiver", self.receiver.to_string());
        if let Some(receiver_time) = &self.aprs.timestamp {
            elements.insert("receiver_time", receiver_time.to_string());
        }
        elements.insert("comment", self.aprs.comment.to_string());
        elements.extend(self.comment.get_elements());
        if let Some(receiver_ts) = self.receiver_ts {
            elements.insert("receiver_ts", receiver_ts.to_string());
        };

        elements
    }
}

#[derive(Debug)]
pub enum OGNPacket {
    Invalid(OGNPacketInvalid),
    Unknown(OGNPacketUnknown),
    Position(OGNPacketPosition),
    Status(OGNPacketStatus),
}

impl OGNPacket {
    pub fn new(ts: DateTime<Utc>, raw_message: &str) -> Self {
        match raw_message.parse::<AprsPacket>() {
            Ok(aprs) => match aprs.data {
                AprsData::Position(position) => {
                    let comment: PositionComment = position.comment.as_str().into();
                    let additional_precision =
                        &comment.additional_precision.clone().unwrap_or_default();
                    let mut packet_position = OGNPacketPosition {
                        ts,
                        raw_message: raw_message.into(),
                        src_call: aprs.from.call,
                        dst_call: aprs.to.call,
                        receiver: aprs.via.iter().last().unwrap().to_string(),
                        aprs: position,
                        comment,

                        receiver_ts: None,
                        relation: None,
                        normalized_quality: None,
                    };

                    *packet_position.aprs.latitude +=
                        (additional_precision.lat as f64) / 1000.0 / 60.0;
                    *packet_position.aprs.longitude +=
                        (additional_precision.lon as f64) / 1000.0 / 60.0;

                    packet_position.receiver_ts = packet_position
                        .aprs
                        .timestamp
                        .as_ref()
                        .and_then(|timestamp| timestamp.guess_date_time(&ts));

                    if packet_position.aprs.timestamp.is_none() {
                        OGNPacket::Invalid(OGNPacketInvalid {
                            ts,
                            raw_message: raw_message.into(),
                            error_message: "Missing timestamp".into(),
                        })
                    } else {
                        OGNPacket::Position(packet_position)
                    }
                }
                AprsData::Status(status) => {
                    let comment = status.comment.as_str().into();
                    let mut packet_status = OGNPacketStatus {
                        ts,
                        raw_message: raw_message.into(),
                        src_call: aprs.from.call,
                        dst_call: aprs.to.call,
                        receiver: aprs.via.iter().last().unwrap().to_string(),
                        aprs: status,
                        comment,

                        receiver_ts: None,
                    };
                    packet_status.receiver_ts = packet_status
                        .aprs
                        .timestamp
                        .as_ref()
                        .and_then(|receiver_ts| receiver_ts.guess_date_time(&ts));

                    if packet_status.aprs.timestamp.is_none() {
                        OGNPacket::Invalid(OGNPacketInvalid {
                            ts,
                            raw_message: raw_message.into(),
                            error_message: "Missing timestamp".into(),
                        })
                    } else {
                        OGNPacket::Status(packet_status)
                    }
                }
                AprsData::Message(_) | AprsData::Unknown => OGNPacket::Unknown(OGNPacketUnknown {
                    ts,
                    raw_message: raw_message.into(),
                    src_call: aprs.from.call,
                    dst_call: aprs.to.call,
                    receiver: aprs.via.iter().last().unwrap().to_string(),
                }),
            },
            Err(err) => OGNPacket::Invalid(OGNPacketInvalid {
                ts,
                raw_message: raw_message.into(),
                error_message: err.to_string(),
            }),
        }
    }
}

mod tests {
    use chrono::TimeZone;
    use ogn_packet::*;
    use crate::*;

    #[test]
    fn test_valid() {
        let position_packet = OGNPacket::new(
            Utc.with_ymd_and_hms(2023, 03, 18, 13, 0, 0).unwrap(),
            "ICAA3D16A>APRS,qAS,Ottobrun2:/120155h4755.05N/01124.85E'056/131/A=003516 !W64! id05A3D16A -157fpm -0.1rot 4.0dB 0e +10.8kHz gps2x2".into());

        if let OGNPacket::Position(pos) = position_packet {
            assert_eq!(*pos.aprs.latitude, 47.9176);
            assert_eq!(*pos.aprs.longitude, 11.414233333333334);
        };
    }

    #[test]
    fn test_invalids() {
        let invalid_packet = OGNPacketInvalid {
            ts: Utc.with_ymd_and_hms(2017, 04, 02, 12, 50, 32).unwrap(),
            raw_message: "This is a \"raw\" message!".into(),
            error_message: "What are you doing with my \\ ?".into(),
        };
        assert_eq!(OGNPacketInvalid::to_ilp("invalids", invalid_packet.get_tags(), invalid_packet.get_fields(), invalid_packet.ts), "invalids raw_message=\"This is a \\\"raw\\\" message!\",error_message=\"What are you doing with my \\\\ ?\" 1491137432000000000\n".to_string());
    }
}
