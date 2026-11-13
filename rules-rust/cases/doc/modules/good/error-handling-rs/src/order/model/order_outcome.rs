#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OrderOutcome {
    Placed,
    Refused,
}
