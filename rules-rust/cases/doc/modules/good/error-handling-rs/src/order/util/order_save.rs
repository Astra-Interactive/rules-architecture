use crate::order::model::{Order, OrderError};

pub(crate) fn save_order(order: Order) -> Result<(), OrderError> {
    if order.items.is_empty() {
        return Err(OrderError::Validation { id: order.id });
    }
    Ok(())
}
