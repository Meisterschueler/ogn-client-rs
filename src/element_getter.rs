use std::collections::HashMap;

use ogn_parser::{
    AprsData, AprsError, AprsPacket, AprsPosition, AprsStatus, Comment, PositionComment,
    ServerComment, ServerResponse, StatusComment,
};
use serde_json::{Value, json};

use crate::messages::server_response_container::ServerResponseContainer;

pub trait ElementGetter {
    fn get_elements(&self) -> HashMap<&str, Value>;
}

impl ElementGetter for PositionComment {
    fn get_elements(&self) -> HashMap<&str, Value> {
        let mut elements: HashMap<&str, Value> = HashMap::new();

        if let Some(course) = self.course {
            elements.insert("course", json!(course));
        };
        if let Some(speed) = self.speed {
            elements.insert("speed", json!(speed));
        };
        if let Some(altitude) = self.altitude {
            elements.insert("altitude", json!(altitude));
        };
        if let Some(additional_precision) = &self.additional_precision {
            elements.insert("additional_lat", json!(additional_precision.lat));
            elements.insert("additional_lon", json!(additional_precision.lon));
        };
        if let Some(id) = &self.id {
            elements.insert("address_type", json!(id.address_type));
            elements.insert("aircraft_type", json!(id.aircraft_type));
            elements.insert("is_stealth", json!(id.is_stealth));
            elements.insert("is_notrack", json!(id.is_notrack));
            elements.insert("address", json!(id.address));
        };

        if let Some(climb_rate) = self.climb_rate {
            elements.insert("climb_rate", json!(climb_rate));
        };
        if let Some(turn_rate) = self.turn_rate {
            elements.insert("turn_rate", json!(turn_rate));
        };
        if let Some(signal_quality) = self.signal_quality {
            elements.insert("signal_quality", json!(signal_quality));
        };
        if let Some(error) = self.error {
            elements.insert("error", json!(error));
        };
        if let Some(frequency_offset) = self.frequency_offset {
            elements.insert("frequency_offset", json!(frequency_offset));
        };
        if let Some(gps_quality) = &self.gps_quality {
            elements.insert("gps_quality", json!(gps_quality));
        };
        if let Some(flight_level) = self.flight_level {
            elements.insert("flight_level", json!(flight_level));
        };
        if let Some(signal_power) = self.signal_power {
            elements.insert("signal_power", json!(signal_power));
        };
        if let Some(software_version) = self.software_version {
            elements.insert("software_version", json!(software_version));
        };
        if let Some(hardware_version) = self.hardware_version {
            elements.insert("hardware_version", json!(hardware_version));
        };
        if let Some(original_address) = self.original_address {
            elements.insert("original_address", json!(original_address));
        };
        if let Some(unparsed) = &self.unparsed {
            elements.insert("unparsed", json!(unparsed));
        };

        elements
    }
}

impl ElementGetter for AprsPosition {
    fn get_elements(&self) -> HashMap<&str, Value> {
        let mut elements = self.comment.get_elements();

        if let Some(timestamp) = &self.timestamp {
            elements.insert("receiver_time", json!(timestamp));
        }

        elements.insert("messageing_supported", json!(self.messaging_supported));
        elements.insert("latitude", json!(self.latitude));
        elements.insert("longitude", json!(self.longitude));
        elements.insert("symbol_table", json!(self.symbol_table));
        elements.insert("symbol_code", json!(self.symbol_code));

        elements
    }
}

impl ElementGetter for StatusComment {
    fn get_elements(&self) -> std::collections::HashMap<&str, Value> {
        let mut elements: HashMap<&str, Value> = HashMap::new();
        if let Some(version) = &self.version {
            elements.insert("version", json!(version));
        };
        if let Some(platform) = &self.platform {
            elements.insert("platform", json!(platform));
        };
        if let Some(cpu_load) = self.cpu_load {
            elements.insert("cpu_load", json!(cpu_load));
        };
        if let Some(ram_free) = self.ram_free {
            elements.insert("ram_free", json!(ram_free));
        };
        if let Some(ram_total) = self.ram_total {
            elements.insert("ram_total", json!(ram_total));
        };
        if let Some(ntp_offset) = self.ntp_offset {
            elements.insert("ntp_offset", json!(ntp_offset));
        };
        if let Some(ntp_correction) = self.ntp_correction {
            elements.insert("ntp_correction", json!(ntp_correction));
        };
        if let Some(voltage) = self.voltage {
            elements.insert("voltage", json!(voltage));
        };
        if let Some(amperage) = self.amperage {
            elements.insert("amperage", json!(amperage));
        };
        if let Some(cpu_temperature) = self.cpu_temperature {
            elements.insert("cpu_temperature", json!(cpu_temperature));
        };
        if let Some(visible_senders) = self.visible_senders {
            elements.insert("visible_senders", json!(visible_senders));
        };
        if let Some(latency) = self.latency {
            elements.insert("latency", json!(latency));
        };
        if let Some(senders) = self.senders {
            elements.insert("senders", json!(senders));
        };
        if let Some(rf_correction_manual) = self.rf_correction_manual {
            elements.insert("rf_correction_manual", json!(rf_correction_manual));
        };
        if let Some(rf_correction_automatic) = self.rf_correction_automatic {
            elements.insert("rf_correction_automatic", json!(rf_correction_automatic));
        };
        if let Some(noise) = self.noise {
            elements.insert("noise", json!(noise));
        };
        if let Some(senders_signal_quality) = self.senders_signal_quality {
            elements.insert("senders_signal_quality", json!(senders_signal_quality));
        };
        if let Some(senders_messages) = self.senders_messages {
            elements.insert("senders_messages", json!(senders_messages));
        };
        if let Some(good_senders_signal_quality) = self.good_senders_signal_quality {
            elements.insert(
                "good_senders_signal_quality",
                json!(good_senders_signal_quality),
            );
        };
        if let Some(good_senders) = self.good_senders {
            elements.insert("good_senders", json!(good_senders));
        };
        if let Some(good_and_bad_senders) = self.good_and_bad_senders {
            elements.insert("good_and_bad_senders", json!(good_and_bad_senders));
        };
        if let Some(unparsed) = &self.unparsed {
            elements.insert("unparsed", json!(unparsed));
        };

        elements
    }
}

impl ElementGetter for AprsStatus {
    fn get_elements(&self) -> HashMap<&str, Value> {
        let mut elements = self.comment.get_elements();

        if let Some(timestamp) = &self.timestamp {
            elements.insert("receiver_time", json!(timestamp));
        }

        elements
    }
}

impl ElementGetter for AprsPacket {
    fn get_elements(&self) -> HashMap<&str, Value> {
        let mut elements: HashMap<&str, Value> = match &self.data {
            AprsData::Position(position) => position.get_elements(),
            AprsData::Status(status) => status.get_elements(),
            AprsData::Message(_) => HashMap::new(),
            AprsData::Unknown => HashMap::new(),
        };

        elements.insert("src_call", json!(self.from));
        elements.insert("dst_call", json!(self.to));
        elements.insert("receiver", json!(self.via));

        elements
    }
}

impl ElementGetter for ServerComment {
    fn get_elements(&self) -> HashMap<&str, Value> {
        let mut elements: HashMap<&str, Value> = HashMap::new();

        elements.insert("version", json!(self.version));
        elements.insert("timestamp", json!(self.timestamp));
        elements.insert("server", json!(self.server));
        elements.insert("ip_address", json!(self.ip_address));
        elements.insert("port", json!(self.port));

        elements
    }
}

impl ElementGetter for AprsError {
    fn get_elements(&self) -> HashMap<&str, Value> {
        let mut elements: HashMap<&str, Value> = HashMap::new();

        elements.insert("error", json!(self));

        elements
    }
}

impl ElementGetter for Comment {
    fn get_elements(&self) -> HashMap<&str, Value> {
        let mut elements: HashMap<&str, Value> = HashMap::new();

        elements.insert("comment", json!(self.comment));

        elements
    }
}

impl ElementGetter for ServerResponse {
    fn get_elements(&self) -> HashMap<&str, Value> {
        match self {
            ServerResponse::AprsPacket(aprs_packet) => aprs_packet.get_elements(),
            ServerResponse::ServerComment(server_comment) => server_comment.get_elements(),
            ServerResponse::ParserError(parser_error) => parser_error.get_elements(),
            ServerResponse::Comment(comment) => comment.get_elements(),
        }
    }
}

impl ElementGetter for ServerResponseContainer {
    fn get_elements(&self) -> HashMap<&str, Value> {
        let mut elements = self.server_response.get_elements();

        elements.insert("ts", json!(self.ts));
        elements.insert("raw_message", json!(self.raw_message));
        if let Some(receiver_ts) = &self.receiver_ts {
            elements.insert("receiver_ts", json!(receiver_ts));
        }
        if let Some(bearing) = &self.bearing {
            elements.insert("bearing", json!(bearing));
        }
        if let Some(distance) = &self.distance {
            elements.insert("distance", json!(distance));
        }
        if let Some(normalized_signal_quality) = &self.normalized_signal_quality {
            elements.insert(
                "normalized_signal_quality",
                json!(normalized_signal_quality),
            );
        }
        if let Some(plausibility) = &self.plausibility {
            elements.insert("plausibility", json!(plausibility));
        }

        elements
    }
}
