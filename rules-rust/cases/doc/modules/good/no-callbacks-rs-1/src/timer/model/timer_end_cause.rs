#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimerEndCause {
    Elapsed,
    Cancelled,
}
