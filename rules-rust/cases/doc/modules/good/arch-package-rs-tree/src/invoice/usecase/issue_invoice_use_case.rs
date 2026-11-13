use crate::invoice::api::InvoiceRepository;
use crate::invoice::model::{Invoice, InvoiceError};
use std::sync::Arc;

pub struct IssueInvoiceUseCase {
    invoice_repository: Arc<dyn InvoiceRepository>,
}

impl IssueInvoiceUseCase {
    pub fn new(invoice_repository: Arc<dyn InvoiceRepository>) -> Self {
        Self { invoice_repository }
    }

    pub fn invoke(&self, number: u32) -> Result<Option<Invoice>, InvoiceError> {
        let invoice = self.invoice_repository.find_invoice(number)?;
        Ok(invoice.filter(Invoice::is_numbered))
    }
}
