use crate::order::api::OrderRepository;
use crate::order::model::CreateOrderCommand;
use async_trait::async_trait;
use parking_lot::Mutex;

pub(crate) struct FakeOrderRepository {
    saved: Mutex<Vec<CreateOrderCommand>>,
}

impl FakeOrderRepository {
    pub(crate) fn new() -> Self {
        Self {
            saved: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl OrderRepository for FakeOrderRepository {
    async fn save_order(&self, command: CreateOrderCommand) {
        self.saved.lock().push(command);
    }
}
