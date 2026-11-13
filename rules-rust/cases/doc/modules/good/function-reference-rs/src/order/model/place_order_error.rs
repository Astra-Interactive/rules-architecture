use crate::order::model::OrderError;

#[derive(Debug, thiserror::Error)]
pub(crate) enum PlaceOrderError {
    #[error(transparent)]
    Order(#[from] OrderError),
}
