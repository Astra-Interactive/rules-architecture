// clock/system/system_clock.rs
use crate::clock::api::Clock;
use std::time::SystemTime;

pub(crate) struct SystemClock;

impl Clock for SystemClock {
    #[expect(
        clippy::disallowed_methods,
        reason = "SystemClock is the one reader of the system clock"
    )]
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}
