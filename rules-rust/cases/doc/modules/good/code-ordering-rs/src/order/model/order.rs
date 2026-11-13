use crate::order::model::Money;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Order {
    pub(crate) items: Vec<u64>,
    pub(crate) subtotal: Money,
}
