use crate::order::model::{Order, OrderError};

pub(crate) fn parse_order(raw: &str) -> Result<Order, OrderError> {
    if raw.trim().is_empty() {
        return Err(OrderError::Blank);
    }
    Ok(Order {
        raw_length: raw.len(),
    })
}
