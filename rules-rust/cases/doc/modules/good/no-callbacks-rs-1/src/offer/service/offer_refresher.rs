use crate::offer::memory::InMemoryOfferCache;
use crate::task::service::TaskScope;
use std::sync::Arc;
use std::time::Duration;

pub(crate) struct OfferRefresher {
    offer_cache: Arc<InMemoryOfferCache>,
    task_scope: Arc<TaskScope>,
    interval: Duration,
}

impl OfferRefresher {
    pub(crate) fn new(
        offer_cache: Arc<InMemoryOfferCache>,
        task_scope: Arc<TaskScope>,
        interval: Duration,
    ) -> Self {
        Self {
            offer_cache,
            task_scope,
            interval,
        }
    }

    pub(crate) fn start(&self) {
        let offer_cache = Arc::clone(&self.offer_cache);
        let interval = self.interval;
        self.task_scope.spawn(async move {
            let mut ticks = tokio::time::interval(interval);
            loop {
                ticks.tick().await;
                offer_cache.refresh().await;
            }
        });
    }
}
