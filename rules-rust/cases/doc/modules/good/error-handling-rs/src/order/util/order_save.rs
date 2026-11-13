use crate::order::model::{Order, OrderError};

pub(crate) fn save_order(order: Order) -> Result<(), OrderError> {
    if order.closed {
        return Err(OrderError::Closed { id: order.id });
    }
    Ok(())
}
