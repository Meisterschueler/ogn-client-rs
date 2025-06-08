use actix::prelude::*;
use chrono::prelude::*;
use ogn_parser::ServerResponse;

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
