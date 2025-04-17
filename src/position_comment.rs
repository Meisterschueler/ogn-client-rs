use std::collections::HashMap;

use ogn_parser::PositionComment;

use crate::ogn_packet::ElementGetter;

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
