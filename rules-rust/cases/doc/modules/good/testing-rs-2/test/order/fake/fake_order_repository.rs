use crate::order::api::OrderRepository;

pub(crate) struct FakeOrderRepository {
    count: usize,
}

impl FakeOrderRepository {
    pub(crate) fn new(count: usize) -> Self {
        Self { count }
    }
}

impl OrderRepository for FakeOrderRepository {
    fn count_orders(&self) -> usize {
        self.count
    }
}
