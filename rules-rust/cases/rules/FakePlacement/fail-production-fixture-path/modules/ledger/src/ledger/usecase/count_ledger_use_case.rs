use case_billing_api::fixture::invoice::fake::FakeInvoiceRepository;
use case_billing_api::invoice::api::InvoiceRepository;
use std::sync::Arc;

pub struct CountLedgerUseCase {
    invoice_repository: Arc<dyn InvoiceRepository>,
}

impl CountLedgerUseCase {
    pub fn new(invoice_repository: Arc<dyn InvoiceRepository>) -> Self {
        Self { invoice_repository }
    }

    pub fn with_defaults() -> Self {
        Self::new(Arc::new(FakeInvoiceRepository::new(0)))
    }

    pub fn invoke(&self) -> u32 {
        self.invoice_repository.count().max(1)
    }
}
