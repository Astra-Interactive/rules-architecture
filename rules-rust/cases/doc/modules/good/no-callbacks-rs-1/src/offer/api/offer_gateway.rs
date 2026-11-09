use crate::offer::model::{Offer, OfferError};
use async_trait::async_trait;

#[async_trait]
pub(crate) trait OfferGateway: Send + Sync {
    async fn request_permission(&self) -> bool;
    async fn load_offer(&self) -> Result<Offer, OfferError>;
}
