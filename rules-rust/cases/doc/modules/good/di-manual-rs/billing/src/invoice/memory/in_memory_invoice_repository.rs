use crate::invoice::api::InvoiceRepository;
use parking_lot::Mutex;
use std::collections::HashSet;

pub(crate) struct InMemoryInvoiceRepository {
    paid: Mutex<HashSet<u32>>,
}

impl InMemoryInvoiceRepository {
    pub(crate) fn new() -> Self {
        Self {
            paid: Mutex::new(HashSet::new()),
        }
    }
}

impl InvoiceRepository for InMemoryInvoiceRepository {
    fn mark_paid(&self, number: u32) -> bool {
        self.paid.lock().insert(number)
    }
}
