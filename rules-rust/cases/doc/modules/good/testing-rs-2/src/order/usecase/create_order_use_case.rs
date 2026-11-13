use crate::order::api::OrderRepository;
use std::sync::Arc;

pub(crate) struct CreateOrderUseCase {
    order_repository: Arc<dyn OrderRepository>,
}

impl CreateOrderUseCase {
    pub(crate) fn new(order_repository: Arc<dyn OrderRepository>) -> Self {
        Self { order_repository }
    }

    pub(crate) fn invoke(&self, limit: usize) -> bool {
        let count = self.order_repository.count_orders();
        count < limit
    }
}
