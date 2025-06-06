use chrono::prelude::*;
use ogn_parser::{AdditionalPrecision, Callsign, Latitude, Longitude, Timestamp};
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct StatusContainer {
    // Fields from ServerResponseContainer
    pub ts: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub raw_message: String,
    pub receiver_ts: Option<DateTime<Utc>>,

    // Fields from AprsPacket
    pub src_call: Callsign,
    pub dst_call: Callsign,
    pub receiver: Option<Callsign>,

    // Fields from AprsStatus
    pub receiver_time: Option<Timestamp>,

    // Fields from StatusComment
    pub version: Option<String>,
    pub platform: Option<String>,
    pub cpu_load: Option<Decimal>,
    pub ram_free: Option<Decimal>,
    pub ram_total: Option<Decimal>,
    pub ntp_offset: Option<Decimal>,
    pub ntp_correction: Option<Decimal>,
    pub voltage: Option<Decimal>,
    pub amperage: Option<Decimal>,
    pub cpu_temperature: Option<Decimal>,
    pub visible_senders: Option<u16>,
    pub latency: Option<Decimal>,
    pub senders: Option<u16>,
    pub rf_correction_manual: Option<i16>,
    pub rf_correction_automatic: Option<Decimal>,
    pub noise: Option<Decimal>,
    pub senders_signal_quality: Option<Decimal>,
    pub senders_messages: Option<u32>,
    pub good_senders_signal_quality: Option<Decimal>,
    pub good_senders: Option<u16>,
    pub good_and_bad_senders: Option<u16>,
    pub unparsed: Option<String>,
}
