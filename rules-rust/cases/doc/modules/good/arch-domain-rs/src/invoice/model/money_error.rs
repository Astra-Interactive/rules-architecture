#[derive(Debug, thiserror::Error)]
pub(crate) enum MoneyError {
    #[error("{raw} is not an amount of cents")]
    NotANumber { raw: String },
    #[error("money is never negative, got {cents}")]
    Negative { cents: i64 },
}
