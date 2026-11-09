use crate::order::model::OrderId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum OrderReply {
    Accepted(OrderId),
    Rejected(String),
}
