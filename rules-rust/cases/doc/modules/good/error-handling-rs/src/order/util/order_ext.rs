// util/order_ext.rs
use crate::order::model::{Order, OrderError};

impl Order {
    pub(crate) fn ensure_open(self) -> Result<Order, OrderError> {
        if self.closed {
            return Err(OrderError::Closed { id: self.id });
        }
        Ok(self)
    }
}
