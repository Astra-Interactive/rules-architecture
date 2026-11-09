#[derive(Debug, thiserror::Error)]
pub(crate) enum InvoiceError {
    #[error("the invoice store is unavailable")]
    Unavailable,
}
