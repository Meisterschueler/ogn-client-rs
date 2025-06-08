use std::time::UNIX_EPOCH;

use chrono::prelude::*;
use influxlp_tools::LineProtocol;
use ogn_parser::{AdditionalPrecision, Callsign, Latitude, Longitude, Timestamp};
use rust_decimal::prelude::*;
use serde::Serialize;

fn serialize_location<S>(pos: &(f64, f64), serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use std::fmt::Write;

    let mut s = String::with_capacity(64);
    write!(&mut s, "SRID=4326;POINT({} {})", pos.0, pos.1).unwrap();
    serializer.serialize_str(&s)
}

#[derive(Debug, Clone, Serialize)]
pub struct PositionContainer {
    // Fields from ServerResponseContainer
    pub ts: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub raw_message: String,
    pub receiver_ts: Option<DateTime<Utc>>,
    pub bearing: Option<f64>,
    pub distance: Option<f64>,
    pub normalized_quality: Option<f64>,
    pub plausibility: Option<u16>,

    // Fields from AprsPacket
    pub src_call: Callsign,
    pub dst_call: Callsign,
    pub receiver: Option<Callsign>,

    // Fields from AprsPosition
    pub receiver_time: Option<Timestamp>,
    #[serde(skip_serializing)]
    pub messaging_supported: bool,
    #[serde(skip_serializing)]
    pub latitude: Latitude,
    #[serde(skip_serializing)]
    pub longitude: Longitude,
    pub symbol_table: char,
    pub symbol_code: char,

    #[serde(serialize_with = "serialize_location")]
    pub location: (f64, f64),

    // Fields from PositionComment
    pub course: Option<u16>,
    pub speed: Option<u16>,
    pub altitude: Option<u32>,
    #[serde(skip_serializing)]
    pub wind_direction: Option<u16>,
    #[serde(skip_serializing)]
    pub wind_speed: Option<u16>,
    #[serde(skip_serializing)]
    pub gust: Option<u16>,
    #[serde(skip_serializing)]
    pub temperature: Option<i16>,
    #[serde(skip_serializing)]
    pub rainfall_1h: Option<u16>,
    #[serde(skip_serializing)]
    pub rainfall_24h: Option<u16>,
    #[serde(skip_serializing)]
    pub rainfall_midnight: Option<u16>,
    #[serde(skip_serializing)]
    pub humidity: Option<u8>,
    #[serde(skip_serializing)]
    pub barometric_pressure: Option<u32>,
    #[serde(skip_serializing)]
    pub additional_precision: Option<AdditionalPrecision>,
    pub climb_rate: Option<i16>,
    pub turn_rate: Option<Decimal>,
    pub signal_quality: Option<Decimal>,
    pub error: Option<u8>,
    pub frequency_offset: Option<Decimal>,
    pub gps_quality: Option<String>,
    pub flight_level: Option<Decimal>,
    pub signal_power: Option<Decimal>,
    pub software_version: Option<Decimal>,
    pub hardware_version: Option<u8>,
    pub original_address: Option<u32>,
    pub unparsed: Option<String>,

    // Fields from ID
    #[serde(skip_serializing)]
    pub reserved: Option<u16>,
    pub address_type: Option<u16>,
    pub aircraft_type: Option<u8>,
    pub is_stealth: Option<bool>,
    pub is_notrack: Option<bool>,
    pub address: Option<u32>,
}

impl PositionContainer {
    pub fn to_ilp(&self) -> String {
        let mut lp = LineProtocol::new("positions");

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
        if let Some(bearing) = self.bearing {
            lp = lp.add_field("bearing", bearing);
        }
        if let Some(distance) = self.distance {
            lp = lp.add_field("distance", distance);
        }
        if let Some(nq) = self.normalized_quality {
            lp = lp.add_field("normalized_quality", nq);
        }
        if let Some(plausibility) = self.plausibility {
            lp = lp.add_field("plausibility", plausibility);
        }

        // Fields from AprsPosition
        if let Some(receiver_time) = &self.receiver_time {
            lp = lp.add_field("receiver_time", receiver_time.to_string());
        }
        lp = lp.add_field("messaging_supported", self.messaging_supported);
        lp = lp.add_field("latitude", *self.latitude);
        lp = lp.add_field("longitude", *self.longitude);
        lp = lp.add_field("symbol_table", self.symbol_table.to_string());
        lp = lp.add_field("symbol_code", self.symbol_code.to_string());

        // Fields from PositionComment
        if let Some(course) = self.course {
            lp = lp.add_field("course", course);
        }
        if let Some(speed) = self.speed {
            lp = lp.add_field("speed", speed);
        }
        if let Some(altitude) = self.altitude {
            lp = lp.add_field("altitude", altitude);
        }
        if let Some(wind_direction) = self.wind_direction {
            lp = lp.add_field("wind_direction", wind_direction);
        }
        if let Some(wind_speed) = self.wind_speed {
            lp = lp.add_field("wind_speed", wind_speed);
        }
        if let Some(gust) = self.gust {
            lp = lp.add_field("gust", gust);
        }
        if let Some(temperature) = self.temperature {
            lp = lp.add_field("temperature", temperature);
        }
        if let Some(rainfall_1h) = self.rainfall_1h {
            lp = lp.add_field("rainfall_1h", rainfall_1h);
        }
        if let Some(rainfall_24h) = self.rainfall_24h {
            lp = lp.add_field("rainfall_24h", rainfall_24h);
        }
        if let Some(rainfall_midnight) = self.rainfall_midnight {
            lp = lp.add_field("rainfall_midnight", rainfall_midnight);
        }
        if let Some(humidity) = self.humidity {
            lp = lp.add_field("humidity", humidity);
        }
        if let Some(barometric_pressure) = self.barometric_pressure {
            lp = lp.add_field("barometric_pressure", barometric_pressure);
        }
        /*if let Some(additional_precision) = &self.additional_precision {
            lp = lp.add_field("additional_precision", additional_precision.to_string());
        }*/
        if let Some(climb_rate) = self.climb_rate {
            lp = lp.add_field("climb_rate", climb_rate);
        }
        if let Some(turn_rate) = &self.turn_rate {
            lp = lp.add_field("turn_rate", turn_rate.to_f64().unwrap());
        }
        if let Some(signal_quality) = &self.signal_quality {
            lp = lp.add_field("signal_quality", signal_quality.to_string());
        }
        if let Some(error) = self.error {
            lp = lp.add_field("error", error);
        }
        if let Some(frequency_offset) = &self.frequency_offset {
            lp = lp.add_field("frequency_offset", frequency_offset.to_f64().unwrap());
        }
        if let Some(gps_quality) = &self.gps_quality {
            lp = lp.add_field("gps_quality", gps_quality);
        }
        if let Some(flight_level) = &self.flight_level {
            lp = lp.add_field("flight_level", flight_level.to_f64().unwrap());
        }
        if let Some(signal_power) = &self.signal_power {
            lp = lp.add_field("signal_power", signal_power.to_f64().unwrap());
        }
        if let Some(software_version) = &self.software_version {
            lp = lp.add_field("software_version", software_version.to_f64().unwrap());
        }
        if let Some(hardware_version) = self.hardware_version {
            lp = lp.add_field("hardware_version", hardware_version);
        }
        if let Some(original_address) = self.original_address {
            lp = lp.add_field("original_address", original_address);
        }
        if let Some(unparsed) = &self.unparsed {
            lp = lp.add_field("unparsed", unparsed);
        }

        // Fields from ID
        if let Some(reserved) = self.reserved {
            lp = lp.add_field("reserved", reserved);
        }
        if let Some(address_type) = self.address_type {
            lp = lp.add_field("address_type", address_type);
        }
        if let Some(aircraft_type) = self.aircraft_type {
            lp = lp.add_field("aircraft_type", aircraft_type);
        }
        if let Some(is_stealth) = self.is_stealth {
            lp = lp.add_field("is_stealth", is_stealth);
        }
        if let Some(is_notrack) = self.is_notrack {
            lp = lp.add_field("is_notrack", is_notrack);
        }
        if let Some(address) = self.address {
            lp = lp.add_field("address", address);
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
