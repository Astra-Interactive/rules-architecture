// model/order_error.rs: the enum and every variant, one file
use crate::order::model::OrderId;
use std::error::Error;

#[derive(Debug, thiserror::Error)]
pub(crate) enum OrderError {
    #[error("order payload is blank")]
    Parse,
    #[error("order {} must contain at least one item", .id.0)]
    Validation { id: OrderId },
    #[error("could not load order {}", .id.0)]
    Load {
        id: OrderId,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },
}
