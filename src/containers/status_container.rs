use std::time::UNIX_EPOCH;

use chrono::prelude::*;
use influxlp_tools::LineProtocol;
use ogn_parser::{Callsign, Timestamp};
use rust_decimal::prelude::*;
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

impl StatusContainer {
    pub fn to_ilp(&self) -> String {
        let mut lp = LineProtocol::new("statuses");

        lp = lp.add_tag("src_call", self.src_call.to_string());
        lp = lp.add_tag("dst_call", self.dst_call.to_string());
        if let Some(receiver) = &self.receiver {
            lp = lp.add_tag("receiver", receiver.to_string());
        }

        // Fields from ServerResponseContainer
        lp = lp.add_field("raw_message", self.raw_message.to_owned());
        if let Some(ts) = self.receiver_ts {
            lp = lp.add_field("receiver_time", ts.to_rfc3339());
        }

        // Fields from AprsStatus
        if let Some(receiver_time) = &self.receiver_time {
            lp = lp.add_field("receiver_time", receiver_time.to_string());
        }

        // Fields from StatusComment
        if let Some(version) = self.version.to_owned() {
            lp = lp.add_field("version", version);
        }
        if let Some(platform) = self.platform.to_owned() {
            lp = lp.add_field("platform", platform);
        }
        if let Some(cpu_load) = self.cpu_load {
            lp = lp.add_field("cpu_load", cpu_load.to_f64().unwrap());
        }
        if let Some(ram_free) = self.ram_free {
            lp = lp.add_field("ram_free", ram_free.to_f64().unwrap());
        }
        if let Some(ram_total) = self.ram_total {
            lp = lp.add_field("ram_total", ram_total.to_f64().unwrap());
        }
        if let Some(ntp_offset) = self.ntp_offset {
            lp = lp.add_field("ntp_offset", ntp_offset.to_f64().unwrap());
        }
        if let Some(ntp_correction) = self.ntp_correction {
            lp = lp.add_field("ntp_correction", ntp_correction.to_f64().unwrap());
        }
        if let Some(voltage) = self.voltage {
            lp = lp.add_field("voltage", voltage.to_f64().unwrap());
        }
        if let Some(amperage) = self.amperage {
            lp = lp.add_field("amperage", amperage.to_f64().unwrap());
        }
        if let Some(cpu_temperature) = self.cpu_temperature {
            lp = lp.add_field("cpu_temperature", cpu_temperature.to_f64().unwrap());
        }
        if let Some(visible_senders) = self.visible_senders {
            lp = lp.add_field("visible_senders", visible_senders);
        }
        if let Some(latency) = self.latency {
            lp = lp.add_field("latency", latency.to_f64().unwrap());
        }
        if let Some(senders) = self.senders {
            lp = lp.add_field("senders", senders);
        }
        if let Some(rf_correction_manual) = self.rf_correction_manual {
            lp = lp.add_field("rf_correction_manual", rf_correction_manual);
        }
        if let Some(rf_correction_automatic) = self.rf_correction_automatic {
            lp = lp.add_field(
                "rf_correction_automatic",
                rf_correction_automatic.to_f64().unwrap(),
            );
        }
        if let Some(noise) = self.noise {
            lp = lp.add_field("noise", noise.to_f64().unwrap());
        }
        if let Some(senders_signal_quality) = self.senders_signal_quality {
            lp = lp.add_field(
                "senders_signal_quality",
                senders_signal_quality.to_f64().unwrap(),
            );
        }
        if let Some(senders_messages) = self.senders_messages {
            lp = lp.add_field("senders_messages", senders_messages);
        }
        if let Some(good_senders_signal_quality) = self.good_senders_signal_quality {
            lp = lp.add_field(
                "good_senders_signal_quality",
                good_senders_signal_quality.to_f64().unwrap(),
            );
        }
        if let Some(good_senders) = self.good_senders {
            lp = lp.add_field("good_senders", good_senders);
        }
        if let Some(good_and_bad_senders) = self.good_and_bad_senders {
            lp = lp.add_field("good_and_bad_senders", good_and_bad_senders);
        }
        if let Some(unparsed) = self.unparsed.to_owned() {
            lp = lp.add_field("unparsed", unparsed);
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
