// util/order_ext.rs
use crate::order::model::{Order, OrderError};

impl Order {
    pub(crate) fn validate(self) -> Result<Order, OrderError> {
        if self.items.is_empty() {
            return Err(OrderError::Validation { id: self.id });
        }
        Ok(self)
    }
}
