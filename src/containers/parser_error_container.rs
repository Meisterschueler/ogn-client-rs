use chrono::prelude::*;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ParserErrorContainer {
    // Fields from ServerResponseContainer
    pub ts: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub raw_message: String,

    pub error_message: String,
}
