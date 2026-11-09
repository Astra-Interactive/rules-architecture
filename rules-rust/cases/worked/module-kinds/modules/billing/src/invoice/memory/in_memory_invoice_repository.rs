use crate::invoice::api::InvoiceRepository;

pub struct InMemoryInvoiceRepository;

impl InvoiceRepository for InMemoryInvoiceRepository {
    fn can_issue_invoice(&self) -> bool {
        true
    }

    fn issue_invoice(&self) {}
}

