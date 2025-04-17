use std::collections::HashMap;

use ogn_parser::StatusComment;

use crate::ogn_packet::ElementGetter;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdr() {
        let result = "v0.2.7.RPI-GPU CPU:0.7 RAM:770.2/968.2MB NTP:1.8ms/-3.3ppm +55.7C 7/8Acfts[1h] RF:+54-1.1ppm/-0.16dB/+7.1dB@10km[19481]/+16.8dB@10km[7/13]".parse::<StatusComment>().unwrap();
        assert_eq!(
            result,
            StatusComment {
                version: Some("0.2.7".into()),
                platform: Some("RPI-GPU".into()),
                cpu_load: Some(0.7),
                ram_free: Some(770.2),
                ram_total: Some(968.2),
                ntp_offset: Some(1.8),
                ntp_correction: Some(-3.3),
                voltage: None,
                amperage: None,
                cpu_temperature: Some(55.7),
                visible_senders: Some(7),
                senders: Some(8),
                rf_correction_manual: Some(54),
                rf_correction_automatic: Some(-1.1),
                noise: Some(-0.16),
                senders_signal_quality: Some(7.1),
                senders_messages: Some(19481),
                good_senders_signal_quality: Some(16.8),
                good_senders: Some(7),
                good_and_bad_senders: Some(13),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_sdr_different_order() {
        let result = "NTP:1.8ms/-3.3ppm +55.7C CPU:0.7 RAM:770.2/968.2MB 7/8Acfts[1h] RF:+54-1.1ppm/-0.16dB/+7.1dB@10km[19481]/+16.8dB@10km[7/13] v0.2.7.RPI-GPU".parse::<StatusComment>().unwrap();
        assert_eq!(
            result,
            StatusComment {
                version: Some("0.2.7".into()),
                platform: Some("RPI-GPU".into()),
                cpu_load: Some(0.7),
                ram_free: Some(770.2),
                ram_total: Some(968.2),
                ntp_offset: Some(1.8),
                ntp_correction: Some(-3.3),
                voltage: None,
                amperage: None,
                cpu_temperature: Some(55.7),
                visible_senders: Some(7),
                senders: Some(8),
                rf_correction_manual: Some(54),
                rf_correction_automatic: Some(-1.1),
                noise: Some(-0.16),
                senders_signal_quality: Some(7.1),
                senders_messages: Some(19481),
                good_senders_signal_quality: Some(16.8),
                good_senders: Some(7),
                good_and_bad_senders: Some(13),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_rf_3() {
        let result = "RF:+29+0.0ppm/+35.22dB".parse::<StatusComment>().unwrap();
        assert_eq!(
            result,
            StatusComment {
                rf_correction_manual: Some(29),
                rf_correction_automatic: Some(0.0),
                noise: Some(35.22),
                ..Default::default()
            }
        )
    }

    #[test]
    fn test_rf_6() {
        let result = "RF:+41+56.0ppm/-1.87dB/+0.1dB@10km[1928]"
            .parse::<StatusComment>()
            .unwrap();
        assert_eq!(
            result,
            StatusComment {
                rf_correction_manual: Some(41),
                rf_correction_automatic: Some(56.0),
                noise: Some(-1.87),
                senders_signal_quality: Some(0.1),
                senders_messages: Some(1928),
                ..Default::default()
            }
        )
    }

    #[test]
    fn test_rf_10() {
        let result = "RF:+54-1.1ppm/-0.16dB/+7.1dB@10km[19481]/+16.8dB@10km[7/13]"
            .parse::<StatusComment>()
            .unwrap();
        assert_eq!(
            result,
            StatusComment {
                rf_correction_manual: Some(54),
                rf_correction_automatic: Some(-1.1),
                noise: Some(-0.16),
                senders_signal_quality: Some(7.1),
                senders_messages: Some(19481),
                good_senders_signal_quality: Some(16.8),
                good_senders: Some(7),
                good_and_bad_senders: Some(13),
                ..Default::default()
            }
        )
    }
}
