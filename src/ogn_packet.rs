use aprs_parser::{AprsData, AprsPacket, AprsPosition, AprsStatus};
use chrono::{DateTime, SecondsFormat, Utc};

use crate::{
    date_time_guesser::DateTimeGuesser, distance_service::Relation, PositionComment, StatusComment,
};

pub trait CsvSerializer {
    fn csv_header() -> String;
    fn to_csv(&self) -> String;
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
        let aprs_csv =
            "receiver_time,messaging_supported,latitude,longitude,symbol_table,symbol_code,comment";
        let comment_csv = PositionComment::csv_header();
        format!("ts,raw_message,src_call,dst_call,receiver,{aprs_csv},{comment_csv},receiver_ts,bearing,distance,normalized_quality,location")
    }

    fn to_csv(&self) -> String {
        format!(
            "\"{}\",\"{}\",{},{},{},{},{},{},{},{},{},\"{}\",{},{},{},{},{},SRID=4326;POINT({} {})",
            self.ts.to_rfc3339_opts(SecondsFormat::Nanos, true),
            self.raw_message.replace('"', "\"\""),
            self.src_call,
            self.dst_call,
            self.receiver,
            self.aprs
                .timestamp
                .as_ref()
                .map(|ts| ts.to_string())
                .unwrap_or_default(),
            self.aprs.messaging_supported,
            *self.aprs.latitude,
            *self.aprs.longitude,
            self.aprs.symbol_table,
            self.aprs.symbol_code,
            self.aprs.comment.replace('"', "\"\""),
            self.comment.to_csv(),
            self.receiver_ts
                .map(|ts| format!("\"{}\"", ts))
                .unwrap_or_default(),
            self.relation
                .map(|val| val.bearing.to_string())
                .unwrap_or_default(),
            self.relation
                .map(|val| val.distance.to_string())
                .unwrap_or_default(),
            self.normalized_quality
                .map(|val| val.to_string())
                .unwrap_or_default(),
            *self.aprs.longitude,
            *self.aprs.latitude
        )
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
        let aprs_csv = "receiver_time,comment";
        let comment_csv = StatusComment::csv_header();
        format!("ts,raw_message,src_call,dst_call,receiver,{aprs_csv},{comment_csv},receiver_ts")
    }

    fn to_csv(&self) -> String {
        format!(
            "\"{}\",\"{}\",{},{},{},{},\"{}\",{},{}",
            self.ts.to_rfc3339_opts(SecondsFormat::Nanos, true),
            self.raw_message.replace('"', "\"\""),
            self.src_call,
            self.dst_call,
            self.receiver,
            self.aprs
                .timestamp
                .as_ref()
                .map(|ts| ts.to_string())
                .unwrap_or_default(),
            self.aprs.comment.replace('"', "\"\""),
            self.comment.to_csv(),
            self.receiver_ts
                .map(|ts| format!("\"{}\"", ts))
                .unwrap_or_default(),
        )
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

                    *packet_position.aprs.latitude += (additional_precision.lat as f64) / 1000.0;
                    *packet_position.aprs.longitude += (additional_precision.lon as f64) / 1000.0;

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
