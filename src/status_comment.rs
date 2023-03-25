use crate::{
    ogn_packet::CsvSerializer,
    utils::{extract_values, split_value_unit},
};

#[derive(Debug, PartialEq, Default)]
pub struct StatusComment {
    pub version: Option<String>,
    pub platform: Option<String>,
    pub cpu_load: Option<f32>,
    pub ram_free: Option<f32>,
    pub ram_total: Option<f32>,
    pub ntp_offset: Option<f32>,
    pub ntp_correction: Option<f32>,
    pub voltage: Option<f32>,
    pub amperage: Option<f32>,
    pub cpu_temperature: Option<f32>,
    pub visible_senders: Option<u16>,
    pub latency: Option<f32>,
    pub senders: Option<u16>,
    pub rf_correction_manual: Option<i16>,
    pub rf_correction_automatic: Option<f32>,
    pub noise: Option<f32>,
    pub senders_signal_quality: Option<f32>,
    pub senders_messages: Option<u32>,
    pub good_senders_signal_quality: Option<f32>,
    pub good_senders: Option<u16>,
    pub good_and_bad_senders: Option<u16>,
    pub unparsed: Option<String>,
}

impl From<&str> for StatusComment {
    fn from(s: &str) -> Self {
        let mut status_comment = StatusComment {
            ..Default::default()
        };
        let mut unparsed: Vec<_> = vec![];
        for part in s.split_whitespace() {
            if &part[0..1] == "v"
                && part.matches('.').count() == 3
                && status_comment.version.is_none()
            {
                let (first, second) = part
                    .match_indices('.')
                    .nth(2)
                    .map(|(idx, _)| part.split_at(idx))
                    .unwrap();
                status_comment.version = Some(first[1..].into());
                status_comment.platform = Some(second[1..].into());
            } else if part.len() > 4
                && part.starts_with("CPU:")
                && status_comment.cpu_load.is_none()
            {
                if let Ok(cpu_load) = part[4..].parse::<f32>() {
                    status_comment.cpu_load = Some(cpu_load);
                } else {
                    unparsed.push(part);
                }
            } else if part.len() > 6
                && part.starts_with("RAM:")
                && part.ends_with("MB")
                && part.find('/').is_some()
                && status_comment.ram_free.is_none()
            {
                let subpart = &part[4..part.len() - 2];
                let split_point = subpart.find('/').unwrap();
                let (first, second) = subpart.split_at(split_point);
                let ram_free = first.parse::<f32>().ok();
                let ram_total = second[1..].parse::<f32>().ok();
                if ram_free.is_some() && ram_total.is_some() {
                    status_comment.ram_free = ram_free;
                    status_comment.ram_total = ram_total;
                } else {
                    unparsed.push(part);
                }
            } else if part.len() > 6
                && part.starts_with("NTP:")
                && part.find('/').is_some()
                && status_comment.ntp_offset.is_none()
            {
                let subpart = &part[4..part.len() - 3];
                let split_point = subpart.find('/').unwrap();
                let (first, second) = subpart.split_at(split_point);
                let ntp_offset = first[0..first.len() - 2].parse::<f32>().ok();
                let ntp_correction = second[1..].parse::<f32>().ok();
                if ntp_offset.is_some() && ntp_correction.is_some() {
                    status_comment.ntp_offset = ntp_offset;
                    status_comment.ntp_correction = ntp_correction;
                } else {
                    unparsed.push(part);
                }
            } else if part.len() >= 11
                && part.ends_with("Acfts[1h]")
                && part.find('/').is_some()
                && status_comment.visible_senders.is_none()
            {
                let subpart = &part[0..part.len() - 9];
                let split_point = subpart.find('/').unwrap();
                let (first, second) = subpart.split_at(split_point);
                let visible_senders = first.parse::<u16>().ok();
                let senders = second[1..].parse::<u16>().ok();
                if visible_senders.is_some() && senders.is_some() {
                    status_comment.visible_senders = visible_senders;
                    status_comment.senders = senders;
                } else {
                    unparsed.push(part);
                }
            } else if part.len() > 5
                && part.starts_with("Lat:")
                && part.ends_with("s")
                && status_comment.latency.is_none()
            {
                let latency = part[4..part.len() - 1].parse::<f32>().ok();
                if latency.is_some() {
                    status_comment.latency = latency;
                } else {
                    unparsed.push(part);
                }
            } else if part.len() >= 11
                && part.starts_with("RF:")
                && status_comment.rf_correction_manual.is_none()
            {
                let values = extract_values(part);

                if values.len() == 10 {
                    let rf_correction_manual = values[0].parse::<i16>().ok();
                    let rf_correction_automatic = values[1].parse::<f32>().ok();
                    let noise = values[2].parse::<f32>().ok();
                    let senders_signal_quality = values[3].parse::<f32>().ok();
                    let senders_messages = values[5].parse::<u32>().ok();
                    let good_senders_signal_quality = values[6].parse::<f32>().ok();
                    let good_senders = values[8].parse::<u16>().ok();
                    let good_and_bad_senders = values[9].parse::<u16>().ok();
                    if rf_correction_manual.is_some()
                        && rf_correction_automatic.is_some()
                        && noise.is_some()
                        && senders_signal_quality.is_some()
                        && senders_messages.is_some()
                        && good_senders_signal_quality.is_some()
                        && good_senders.is_some()
                        && good_and_bad_senders.is_some()
                    {
                        status_comment.rf_correction_manual = rf_correction_manual;
                        status_comment.rf_correction_automatic = rf_correction_automatic;
                        status_comment.noise = noise;
                        status_comment.senders_signal_quality = senders_signal_quality;
                        status_comment.senders_messages = senders_messages;
                        status_comment.good_senders_signal_quality = good_senders_signal_quality;
                        status_comment.good_senders = good_senders;
                        status_comment.good_and_bad_senders = good_and_bad_senders;
                    } else {
                        unparsed.push(part);
                        continue;
                    }
                }
            } else if let Some((value, unit)) = split_value_unit(part) {
                if unit == "C" && status_comment.cpu_temperature.is_none() {
                    status_comment.cpu_temperature = value.parse::<f32>().ok();
                } else if unit == "V" && status_comment.voltage.is_none() {
                    status_comment.voltage = value.parse::<f32>().ok();
                } else if unit == "A" && status_comment.amperage.is_none() {
                    status_comment.amperage = value.parse::<f32>().ok();
                } else {
                    unparsed.push(part);
                }
            } else {
                unparsed.push(part);
            }
        }
        status_comment.unparsed = if !unparsed.is_empty() {
            Some(unparsed.join(" "))
        } else {
            None
        };
        status_comment
    }
}

impl CsvSerializer for StatusComment {
    fn csv_header() -> String {
        "version,platform,cpu_load,ram_free,ram_total,ntp_offset,ntp_correction,voltage,amperage,cpu_temperature,visible_senders,latency,senders,rf_correction_manual,rf_correction_automatic,noise,senders_signal_quality,senders_messages,good_senders_signal_quality,good_senders,good_and_bad_senders,unparsed".to_string()
    }

    fn to_csv(&self) -> String {
        let version = self
            .version
            .as_ref()
            .map(|val| val.to_string())
            .unwrap_or_default();
        let platform = self
            .platform
            .as_ref()
            .map(|val| val.to_string())
            .unwrap_or_default();
        let cpu_load = self.cpu_load.map(|val| val.to_string()).unwrap_or_default();
        let ram_free = self.ram_free.map(|val| val.to_string()).unwrap_or_default();
        let ram_total = self
            .ram_total
            .map(|val| val.to_string())
            .unwrap_or_default();
        let ntp_offset = self
            .ntp_offset
            .map(|val| val.to_string())
            .unwrap_or_default();
        let ntp_correction = self
            .ntp_correction
            .map(|val| val.to_string())
            .unwrap_or_default();
        let visible_senders = self
            .visible_senders
            .map(|val| val.to_string())
            .unwrap_or_default();
        let latency = self.latency.map(|val| val.to_string()).unwrap_or_default();
        let senders = self.senders.map(|val| val.to_string()).unwrap_or_default();
        let rf_correction_manual = self
            .rf_correction_manual
            .map(|val| val.to_string())
            .unwrap_or_default();
        let rf_correction_automatic = self
            .rf_correction_automatic
            .map(|val| val.to_string())
            .unwrap_or_default();
        let voltage = self.voltage.map(|val| val.to_string()).unwrap_or_default();
        let amperage = self.amperage.map(|val| val.to_string()).unwrap_or_default();
        let cpu_temperature = self
            .cpu_temperature
            .map(|val| val.to_string())
            .unwrap_or_default();
        let noise = self.noise.map(|val| val.to_string()).unwrap_or_default();
        let senders_signal_quality = self
            .senders_signal_quality
            .map(|val| val.to_string())
            .unwrap_or_default();
        let senders_messages = self
            .senders_messages
            .map(|val| val.to_string())
            .unwrap_or_default();
        let good_senders_signal_quality = self
            .good_senders_signal_quality
            .map(|val| val.to_string())
            .unwrap_or_default();
        let good_senders = self
            .good_senders
            .map(|val| val.to_string())
            .unwrap_or_default();
        let good_and_bad_senders = self
            .good_and_bad_senders
            .map(|val| val.to_string())
            .unwrap_or_default();
        let unparsed = &self
            .unparsed
            .clone()
            .map(|val| val.replace('"', "\"\""))
            .unwrap_or_default();

        format!(
            "{version},{platform},{cpu_load},{ram_free},{ram_total},{ntp_offset},{ntp_correction},{voltage},{amperage},{cpu_temperature},{visible_senders},{latency},{senders},{rf_correction_manual},{rf_correction_automatic},{noise},{senders_signal_quality},{senders_messages},{good_senders_signal_quality},{good_senders},{good_and_bad_senders},\"{unparsed}\""
        )
    }

    fn get_fields(&self) -> Vec<(&str, String)> {
        let mut fields = vec![];
        if let Some(version) = &self.version {
            fields.push(("version", version.clone()))
        };
        if let Some(platform) = &self.platform {
            fields.push(("platform", platform.clone()))
        };
        if let Some(cpu_load) = self.cpu_load {
            fields.push(("cpu_load", cpu_load.to_string()))
        };
        if let Some(ram_free) = self.ram_free {
            fields.push(("ram_free", ram_free.to_string()))
        };
        if let Some(ram_total) = self.ram_total {
            fields.push(("ram_total", ram_total.to_string()))
        };
        if let Some(ntp_offset) = self.ntp_offset {
            fields.push(("ntp_offset", ntp_offset.to_string()))
        };
        if let Some(ntp_correction) = self.ntp_correction {
            fields.push(("ntp_correction", ntp_correction.to_string()))
        };
        if let Some(voltage) = self.voltage {
            fields.push(("voltage", voltage.to_string()))
        };
        if let Some(amperage) = self.amperage {
            fields.push(("amperage", amperage.to_string()))
        };
        if let Some(cpu_temperature) = self.cpu_temperature {
            fields.push(("cpu_temperature", cpu_temperature.to_string()))
        };
        if let Some(visible_senders) = self.visible_senders {
            fields.push(("visible_senders", visible_senders.to_string()))
        };
        if let Some(latency) = self.latency {
            fields.push(("latency", latency.to_string()))
        };
        if let Some(senders) = self.senders {
            fields.push(("senders", senders.to_string()))
        };
        if let Some(rf_correction_manual) = self.rf_correction_manual {
            fields.push(("rf_correction_manual", rf_correction_manual.to_string()))
        };
        if let Some(rf_correction_automatic) = self.rf_correction_automatic {
            fields.push((
                "rf_correction_automatic",
                rf_correction_automatic.to_string(),
            ))
        };
        if let Some(noise) = self.noise {
            fields.push(("noise", noise.to_string()))
        };
        if let Some(senders_signal_quality) = self.senders_signal_quality {
            fields.push(("senders_signal_quality", senders_signal_quality.to_string()))
        };
        if let Some(senders_messages) = self.senders_messages {
            fields.push(("senders_messages", senders_messages.to_string()))
        };
        if let Some(good_senders_signal_quality) = self.good_senders_signal_quality {
            fields.push((
                "good_senders_signal_quality",
                good_senders_signal_quality.to_string(),
            ))
        };
        if let Some(good_senders) = self.good_senders {
            fields.push(("good_senders", good_senders.to_string()))
        };
        if let Some(good_and_bad_senders) = self.good_and_bad_senders {
            fields.push(("good_and_bad_senders", good_and_bad_senders.to_string()))
        };
        if let Some(unparsed) = &self.unparsed {
            fields.push(("unparsed", unparsed.clone()))
        };

        fields
    }

    fn get_tags(&self) -> Vec<(&str, String)> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdr() {
        let result: StatusComment = "v0.2.7.RPI-GPU CPU:0.7 RAM:770.2/968.2MB NTP:1.8ms/-3.3ppm +55.7C 7/8Acfts[1h] RF:+54-1.1ppm/-0.16dB/+7.1dB@10km[19481]/+16.8dB@10km[7/13]".into();
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
        let result: StatusComment = "NTP:1.8ms/-3.3ppm +55.7C CPU:0.7 RAM:770.2/968.2MB 7/8Acfts[1h] RF:+54-1.1ppm/-0.16dB/+7.1dB@10km[19481]/+16.8dB@10km[7/13] v0.2.7.RPI-GPU".into();
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
}
