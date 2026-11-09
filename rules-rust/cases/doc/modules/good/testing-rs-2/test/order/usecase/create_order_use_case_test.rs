use crate::order::usecase::CreateOrderUseCase;
use crate::test::order::fake::FakeOrderRepository;
use std::sync::Arc;

#[test]
fn given_full_book_when_order_is_created_then_it_is_refused() {
    let create_order_use_case = CreateOrderUseCase::new(Arc::new(FakeOrderRepository::new(3)));

    assert!(!create_order_use_case.invoke(3));
}
