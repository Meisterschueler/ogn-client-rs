use actix::prelude::*;
use chrono::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct OGNMessageWithTimestamp {
    pub ts: DateTime<Utc>,
    pub raw: String,
}
