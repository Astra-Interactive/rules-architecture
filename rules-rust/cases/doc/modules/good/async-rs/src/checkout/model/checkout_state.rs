use crate::checkout::model::{CheckoutStage, OrderId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckoutState {
    pub(crate) order_id: OrderId,
    pub(crate) stage: CheckoutStage,
    pub(crate) is_busy: bool,
}
