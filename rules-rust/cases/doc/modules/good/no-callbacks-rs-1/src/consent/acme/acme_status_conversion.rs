use crate::consent::model::ConsentStatus;
use acme_sdk::AcmeStatus;

impl From<AcmeStatus> for ConsentStatus {
    fn from(status: AcmeStatus) -> Self {
        if status.granted {
            return ConsentStatus::Granted;
        }
        ConsentStatus::Denied
    }
}
