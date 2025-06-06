use chrono::prelude::*;
use ogn_parser::{AdditionalPrecision, Callsign, Latitude, Longitude, Timestamp};
use rust_decimal::Decimal;
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
