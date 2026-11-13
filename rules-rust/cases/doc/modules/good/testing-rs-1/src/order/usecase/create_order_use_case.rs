use crate::order::api::OrderRepository;
use crate::order::model::{CreateOrderCommand, CreateOrderResult};
use crate::payment::api::PaymentGateway;
use std::sync::Arc;

pub(crate) struct CreateOrderUseCase {
    order_repository: Arc<dyn OrderRepository>,
    payment_gateway: Arc<dyn PaymentGateway>,
}

impl CreateOrderUseCase {
    pub(crate) fn new(
        order_repository: Arc<dyn OrderRepository>,
        payment_gateway: Arc<dyn PaymentGateway>,
    ) -> Self {
        Self {
            order_repository,
            payment_gateway,
        }
    }

    pub(crate) async fn invoke(&self, command: CreateOrderCommand) -> CreateOrderResult {
        if command.items.is_empty() {
            return CreateOrderResult::CartIsEmpty;
        }
        if !self.payment_gateway.charge(command.customer_id).await {
            return CreateOrderResult::PaymentDeclined;
        }
        self.order_repository.save_order(command).await;
        CreateOrderResult::Created
    }
}
