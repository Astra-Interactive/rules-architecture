use std::time::{Duration, SystemTime};

pub(crate) struct Session {
    created_at: SystemTime,
    expires_at: SystemTime,
    timeout: Duration,
}
