use crate::invoice::model::{Invoice, InvoiceId};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::time::SystemTime;

pub(crate) struct InMemoryInvoiceCache {
    cache: Mutex<HashMap<InvoiceId, CachedInvoice>>,
}

impl InMemoryInvoiceCache {
    pub(crate) fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) fn store(&self, invoice: Invoice, now: SystemTime) {
        let mut cache = self.cache.lock();
        cache.insert(
            invoice.id,
            CachedInvoice {
                invoice,
                refreshed_at: now,
            },
        );
    }
}

pub(crate) struct CachedInvoice {
    invoice: Invoice,
    refreshed_at: SystemTime,
}
