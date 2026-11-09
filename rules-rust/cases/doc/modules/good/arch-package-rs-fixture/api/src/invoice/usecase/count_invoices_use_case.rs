use crate::invoice::api::InvoiceRepository;
use std::sync::Arc;

pub struct CountInvoicesUseCase {
    invoice_repository: Arc<dyn InvoiceRepository>,
}

impl CountInvoicesUseCase {
    pub fn new(invoice_repository: Arc<dyn InvoiceRepository>) -> Self {
        Self { invoice_repository }
    }

    pub fn invoke(&self) -> usize {
        self.invoice_repository.count_invoices()
    }
}
