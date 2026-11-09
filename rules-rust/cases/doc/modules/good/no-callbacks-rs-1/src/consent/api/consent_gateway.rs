use crate::consent::model::{ConsentError, ConsentStatus};
use async_trait::async_trait;

#[async_trait]
pub(crate) trait ConsentGateway: Send + Sync {
    async fn request_consent(&self) -> Result<ConsentStatus, ConsentError>;
}
