// consent/acme/acme_consent_gateway.rs, the technology module of the made-up Acme SDK
use crate::consent::acme::AcmeCallbackRegistration;
use crate::consent::api::ConsentGateway;
use crate::consent::model::{ConsentError, ConsentStatus};
use acme_sdk::Acme;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::oneshot;

pub(crate) struct AcmeConsentGateway {
    acme: Arc<Acme>,
}

impl AcmeConsentGateway {
    pub(crate) fn new(acme: Arc<Acme>) -> Self {
        Self { acme }
    }
}

#[async_trait]
impl ConsentGateway for AcmeConsentGateway {
    async fn request_consent(&self) -> Result<ConsentStatus, ConsentError> {
        let (sender, receiver) = oneshot::channel();
        let callback_id = self.acme.add_consent_callback(Box::new(move |outcome| {
            #[expect(
                clippy::let_underscore_must_use,
                reason = "the receiver is gone only when the caller was cancelled"
            )]
            let _ = sender.send(outcome);
        }));
        let _registration = AcmeCallbackRegistration::new(Arc::clone(&self.acme), callback_id);
        self.acme.request_consent();
        match receiver.await {
            Ok(Ok(status)) => Ok(ConsentStatus::from(status)),
            Ok(Err(error)) => Err(ConsentError::Rejected {
                reason: error.message,
            }),
            Err(_) => Err(ConsentError::Abandoned),
        }
    }
}
