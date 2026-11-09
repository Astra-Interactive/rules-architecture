use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReminderSchedule {
    pub(crate) every: Duration,
}
