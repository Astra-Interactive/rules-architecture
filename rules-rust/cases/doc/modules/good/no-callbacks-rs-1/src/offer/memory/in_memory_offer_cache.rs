use crate::offer::model::Offer;
use parking_lot::Mutex;

pub(crate) struct InMemoryOfferCache {
    offers: Mutex<Vec<Offer>>,
}

impl InMemoryOfferCache {
    pub(crate) fn new() -> Self {
        Self {
            offers: Mutex::new(Vec::new()),
        }
    }

    pub(crate) async fn refresh(&self) {
        tokio::task::yield_now().await;
        self.offers.lock().clear();
    }
}
