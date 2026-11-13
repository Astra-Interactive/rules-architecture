use crate::order::model::CustomerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CreateOrderCommand {
    pub(crate) customer_id: CustomerId,
    pub(crate) items: Vec<u64>,
}
