use std::collections::HashMap;
#[expect(clippy::disallowed_types, reason = "scaffold of the Bad example: the lock it shows")]
use std::sync::{Mutex, PoisonError};
use std::time::SystemTime;

pub(crate) struct Invoice {
    pub(crate) id: u64,
}

pub(crate) struct InvoiceCacheState {
    pub(crate) invoices: HashMap<u64, Invoice>,
    pub(crate) refreshed_at: SystemTime,
}

pub(crate) struct InvoiceCache {
    #[expect(clippy::disallowed_types, reason = "scaffold of the Bad example: the lock it shows")]
    cache: Mutex<InvoiceCacheState>,
}

impl InvoiceCache {
    pub(crate) fn store(&self, invoice: Invoice, now: SystemTime) {
        let mut cache = self.cache.lock().unwrap_or_else(PoisonError::into_inner);
        cache.invoices.insert(invoice.id, invoice);
        cache.refreshed_at = now;
    }
}
