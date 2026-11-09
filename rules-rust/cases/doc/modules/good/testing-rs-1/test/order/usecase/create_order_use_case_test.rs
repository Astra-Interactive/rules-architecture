// test/order/usecase/create_order_use_case_test.rs
use crate::order::model::{CreateOrderCommand, CreateOrderResult, CustomerId};
use crate::order::usecase::CreateOrderUseCase;
use crate::test::order::fake::FakeOrderRepository;
use crate::test::payment::fake::FakePaymentGateway;
use std::sync::Arc;

#[tokio::test]
async fn given_empty_cart_when_order_is_created_then_returns_cart_is_empty() {
    let create_order_use_case = CreateOrderUseCase::new(
        Arc::new(FakeOrderRepository::new()),
        Arc::new(FakePaymentGateway::new()),
    );

    let result = create_order_use_case
        .invoke(CreateOrderCommand {
            customer_id: CustomerId(1),
            items: Vec::new(),
        })
        .await;

    assert_eq!(CreateOrderResult::CartIsEmpty, result);
}

#[tokio::test]
async fn given_existing_active_customer_when_order_is_created_then_order_is_saved() {
    let create_order_use_case = CreateOrderUseCase::new(
        Arc::new(FakeOrderRepository::new()),
        Arc::new(FakePaymentGateway::new()),
    );

    let result = create_order_use_case
        .invoke(CreateOrderCommand {
            customer_id: CustomerId(2),
            items: vec![7],
        })
        .await;

    assert_eq!(CreateOrderResult::Created, result);
}
