use crate::order::model::CustomerId;
use async_trait::async_trait;

#[async_trait]
pub(crate) trait PaymentGateway: Send + Sync {
    async fn charge(&self, customer_id: CustomerId) -> bool;
}
