use crate::invoice::model::InvoiceId;
use std::error::Error;

#[derive(Debug, thiserror::Error)]
pub(crate) enum InvoiceError {
    #[error("could not serialize invoice {}", .id.0)]
    Serialize {
        id: InvoiceId,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },
}
