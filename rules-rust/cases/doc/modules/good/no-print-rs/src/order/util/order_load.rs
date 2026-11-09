use crate::order::model::{Order, OrderError, OrderId};

pub(crate) fn load_order(order_id: OrderId) -> Result<Order, OrderError> {
    if order_id.0 == 0 {
        return Err(OrderError::Unknown { id: order_id });
    }
    Ok(Order { id: order_id })
}
