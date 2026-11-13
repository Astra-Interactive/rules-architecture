use super::invoice::InvoiceId;

#[derive(Debug, thiserror::Error)]
pub(crate) enum InvoiceError {
    #[error("invoice missing")]
    Missing(InvoiceId),
}

