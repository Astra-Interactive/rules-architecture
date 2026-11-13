use crate::order::model::CreateOrderCommand;
use async_trait::async_trait;

#[async_trait]
pub(crate) trait OrderRepository: Send + Sync {
    async fn save_order(&self, command: CreateOrderCommand);
}
