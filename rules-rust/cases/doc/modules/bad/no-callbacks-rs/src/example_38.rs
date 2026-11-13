use std::time::Duration;

pub(crate) struct Offer;

pub(crate) struct OfferError;

pub(crate) trait OfferGateway {
    fn request_permission(&self, on_result: Box<dyn FnOnce(bool) + Send>);
    fn load_offer(&self, on_success: Box<dyn FnOnce(Offer) + Send>, on_failure: Box<dyn FnOnce(OfferError) + Send>);
}

struct RefreshTask { interval: Duration, block: Box<dyn Fn() + Send + Sync> } // calls block back on a timer
