use crate::order::model::{Money, Order, OrderError, OrderId};

pub(crate) fn parse_order(raw: &str) -> Result<Order, OrderError> {
    if raw.trim().is_empty() {
        return Err(OrderError::Parse);
    }
    Ok(Order { id: OrderId(1), total: Money::ZERO, closed: false })
}
