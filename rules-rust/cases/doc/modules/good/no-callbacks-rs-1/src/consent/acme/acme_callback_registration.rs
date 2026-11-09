// consent/acme/acme_callback_registration.rs
use acme_sdk::Acme;
use std::sync::Arc;

pub(crate) struct AcmeCallbackRegistration {
    acme: Arc<Acme>,
    callback_id: u64,
}

impl AcmeCallbackRegistration {
    pub(crate) fn new(acme: Arc<Acme>, callback_id: u64) -> Self {
        Self { acme, callback_id }
    }
}

impl Drop for AcmeCallbackRegistration {
    fn drop(&mut self) {
        self.acme.remove_consent_callback(self.callback_id);
    }
}
