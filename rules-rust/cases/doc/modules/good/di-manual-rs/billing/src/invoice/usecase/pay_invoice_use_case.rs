use crate::invoice::api::InvoiceRepository;
use shop_core::clock::api::Clock;
use std::sync::Arc;
use std::time::SystemTime;

pub(crate) struct PayInvoiceUseCase {
    invoice_repository: Arc<dyn InvoiceRepository>,
    clock: Arc<dyn Clock>,
}

impl PayInvoiceUseCase {
    pub(crate) fn new(
        invoice_repository: Arc<dyn InvoiceRepository>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            invoice_repository,
            clock,
        }
    }

    pub(crate) fn invoke(&self, number: u32) -> Option<SystemTime> {
        let is_paid = self.invoice_repository.mark_paid(number);
        is_paid.then(|| self.clock.now())
    }
}
