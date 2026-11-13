pub(crate) enum Outcome {
    Ready(Payload),
}
pub(crate) struct Payload {
    pub(crate) value: u64,
}
