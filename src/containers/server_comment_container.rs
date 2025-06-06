use std::net::IpAddr;

use chrono::prelude::*;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ServerCommentContainer {
    // Fields from ServerResponseContainer
    pub ts: DateTime<Utc>,
    pub receiver_ts: Option<DateTime<Utc>>,

    // Fields from ServerComment
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub server: String,
    pub ip_address: IpAddr,
    pub port: u16,
}
