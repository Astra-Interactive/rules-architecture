use crate::order::model::OrderId;

#[derive(Debug, thiserror::Error)]
pub(crate) enum OrderError {
    #[error("order {} is unknown", .id.0)]
    Unknown { id: OrderId },
}
