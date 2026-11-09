#[derive(Debug, thiserror::Error)]
pub(crate) enum OrderError {
    #[error("order payload is blank")]
    Blank,
}
