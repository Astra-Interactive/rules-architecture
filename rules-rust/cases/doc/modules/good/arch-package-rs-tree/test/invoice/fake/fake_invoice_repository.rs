use crate::invoice::api::InvoiceRepository;
use crate::invoice::model::{Invoice, InvoiceError};

pub(crate) struct FakeInvoiceRepository {
    invoice: Option<Invoice>,
}

impl FakeInvoiceRepository {
    pub(crate) fn new(invoice: Option<Invoice>) -> Self {
        Self { invoice }
    }
}

impl InvoiceRepository for FakeInvoiceRepository {
    fn find_invoice(&self, _number: u32) -> Result<Option<Invoice>, InvoiceError> {
        Ok(self.invoice.clone())
    }
}
