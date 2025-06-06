use chrono::prelude::*;
use ogn_parser::{AdditionalPrecision, Callsign, Latitude, Longitude, Timestamp};
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ParserErrorContainer {
    // Fields from ServerResponseContainer
    pub ts: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub raw_message: String,

    pub error: String,
}
