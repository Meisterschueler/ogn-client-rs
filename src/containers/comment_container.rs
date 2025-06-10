use chrono::prelude::*;

pub struct CommentContainer {
    // Fields from ServerResponseContainer
    pub ts: DateTime<Utc>,
    pub raw_message: String,
}
