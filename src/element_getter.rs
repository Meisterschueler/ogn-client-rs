use std::collections::HashMap;

use ogn_parser::{PositionComment, StatusComment};

pub trait ElementGetter {
    fn get_elements(&self) -> HashMap<&str, String>;
}

impl ElementGetter for PositionComment {
    fn get_elements(&self) -> HashMap<&str, String> {
        let mut elements: HashMap<&str, String> = HashMap::new();

        if let Some(course) = self.course {
            elements.insert("course", course.to_string());
        };
        if let Some(speed) = self.speed {
            elements.insert("speed", speed.to_string());
        };
        if let Some(altitude) = self.altitude {
            elements.insert("altitude", altitude.to_string());
        };
        if let Some(additional_precision) = &self.additional_precision {
            elements.insert("additional_lat", additional_precision.lat.to_string());
            elements.insert("additional_lon", additional_precision.lon.to_string());
        };
        if let Some(id) = &self.id {
            elements.insert("address_type", id.address_type.to_string());
            elements.insert("aircraft_type", id.aircraft_type.to_string());
            elements.insert("is_stealth", id.is_stealth.to_string());
            elements.insert("is_notrack", id.is_notrack.to_string());
            elements.insert("address", id.address.to_string());
        };

        if let Some(climb_rate) = self.climb_rate {
            elements.insert("climb_rate", climb_rate.to_string());
        };
        if let Some(turn_rate) = self.turn_rate {
            elements.insert("turn_rate", turn_rate.to_string());
        };
        if let Some(signal_quality) = self.signal_quality {
            elements.insert("signal_quality", signal_quality.to_string());
        };
        if let Some(error) = self.error {
            elements.insert("error", error.to_string());
        };
        if let Some(frequency_offset) = self.frequency_offset {
            elements.insert("frequency_offset", frequency_offset.to_string());
        };
        if let Some(gps_quality) = &self.gps_quality {
            elements.insert("gps_quality", gps_quality.clone());
        };
        if let Some(flight_level) = self.flight_level {
            elements.insert("flight_level", flight_level.to_string());
        };
        if let Some(signal_power) = self.signal_power {
            elements.insert("signal_power", signal_power.to_string());
        };
        if let Some(software_version) = self.software_version {
            elements.insert("software_version", software_version.to_string());
        };
        if let Some(hardware_version) = self.hardware_version {
            elements.insert("hardware_version", hardware_version.to_string());
        };
        if let Some(original_address) = self.original_address {
            elements.insert("original_address", original_address.to_string());
        };
        if let Some(unparsed) = &self.unparsed {
            elements.insert("unparsed", unparsed.clone());
        };

        elements
    }
}

impl ElementGetter for StatusComment {
    fn get_elements(&self) -> std::collections::HashMap<&str, String> {
        let mut elements: HashMap<&str, String> = HashMap::new();
        if let Some(version) = &self.version {
            elements.insert("version", version.clone());
        };
        if let Some(platform) = &self.platform {
            elements.insert("platform", platform.clone());
        };
        if let Some(cpu_load) = self.cpu_load {
            elements.insert("cpu_load", cpu_load.to_string());
        };
        if let Some(ram_free) = self.ram_free {
            elements.insert("ram_free", ram_free.to_string());
        };
        if let Some(ram_total) = self.ram_total {
            elements.insert("ram_total", ram_total.to_string());
        };
        if let Some(ntp_offset) = self.ntp_offset {
            elements.insert("ntp_offset", ntp_offset.to_string());
        };
        if let Some(ntp_correction) = self.ntp_correction {
            elements.insert("ntp_correction", ntp_correction.to_string());
        };
        if let Some(voltage) = self.voltage {
            elements.insert("voltage", voltage.to_string());
        };
        if let Some(amperage) = self.amperage {
            elements.insert("amperage", amperage.to_string());
        };
        if let Some(cpu_temperature) = self.cpu_temperature {
            elements.insert("cpu_temperature", cpu_temperature.to_string());
        };
        if let Some(visible_senders) = self.visible_senders {
            elements.insert("visible_senders", visible_senders.to_string());
        };
        if let Some(latency) = self.latency {
            elements.insert("latency", latency.to_string());
        };
        if let Some(senders) = self.senders {
            elements.insert("senders", senders.to_string());
        };
        if let Some(rf_correction_manual) = self.rf_correction_manual {
            elements.insert("rf_correction_manual", rf_correction_manual.to_string());
        };
        if let Some(rf_correction_automatic) = self.rf_correction_automatic {
            elements.insert(
                "rf_correction_automatic",
                rf_correction_automatic.to_string(),
            );
        };
        if let Some(noise) = self.noise {
            elements.insert("noise", noise.to_string());
        };
        if let Some(senders_signal_quality) = self.senders_signal_quality {
            elements.insert("senders_signal_quality", senders_signal_quality.to_string());
        };
        if let Some(senders_messages) = self.senders_messages {
            elements.insert("senders_messages", senders_messages.to_string());
        };
        if let Some(good_senders_signal_quality) = self.good_senders_signal_quality {
            elements.insert(
                "good_senders_signal_quality",
                good_senders_signal_quality.to_string(),
            );
        };
        if let Some(good_senders) = self.good_senders {
            elements.insert("good_senders", good_senders.to_string());
        };
        if let Some(good_and_bad_senders) = self.good_and_bad_senders {
            elements.insert("good_and_bad_senders", good_and_bad_senders.to_string());
        };
        if let Some(unparsed) = &self.unparsed {
            elements.insert("unparsed", unparsed.clone());
        };

        elements
    }
}
