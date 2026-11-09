use crate::order::model::{Money, OrderId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Order {
    pub(crate) id: OrderId,
    pub(crate) items: Vec<u64>,
    pub(crate) total: Money,
}
