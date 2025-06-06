use std::time::UNIX_EPOCH;

use actix::prelude::*;
use chrono::prelude::*;
use influxlp_tools::LineProtocol;
use ogn_parser::{AprsData, ServerResponse};
use serde_json::Value;

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
    fn get_tags(&self) -> Vec<(&str, Value)> {
        let elements = self.get_elements();
        elements
            .iter()
            .filter(|&(k, _)| ["src_call", "dst_call", "receiver"].contains(k))
            .map(|(k, v)| (*k, v.clone()))
            .collect()
    }

    fn get_fields(&self) -> Vec<(&str, Value)> {
        let elements = self.get_elements();
        elements
            .iter()
            .filter(|&(k, _)| {
                ![
                    "src_call",
                    "dst_call",
                    "receiver",
                    "additional_lat",
                    "additional_lon",
                ]
                .contains(k)
            })
            .map(|(k, v)| (*k, v.clone()))
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

        let mut lp = LineProtocol::new(measurement);
        for (key, value) in self.get_tags().into_iter() {
            lp = lp.add_tag(key, value.to_string());
        }

        for (key, value) in self.get_fields().into_iter() {
            lp = lp.add_field(key, value.to_string());
        }
        let lp = lp.with_timestamp(
            self.ts
                .signed_duration_since(DateTime::<Utc>::from(UNIX_EPOCH))
                .num_nanoseconds()
                .unwrap(),
        );

        lp.build().unwrap()
    }
}
