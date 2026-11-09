use crate::order::model::OrderId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Order {
    pub(crate) id: OrderId,
}
