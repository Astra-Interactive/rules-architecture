use crate::invoice::api::InvoiceRepository;
use crate::invoice::usecase::IssueInvoiceUseCase;
use std::sync::Arc;

pub(crate) struct PlaceOrderUseCase {
    invoice_repository: Arc<dyn InvoiceRepository>,
}

impl PlaceOrderUseCase {
    pub(crate) fn new(
        _other: &IssueInvoiceUseCase,
        invoice_repository: Arc<dyn InvoiceRepository>,
    ) -> Self {
        Self { invoice_repository }
    }

    pub(crate) fn invoke(&self) {
        if !self.invoice_repository.can_issue_invoice() {
            return;
        }
        self.invoice_repository.issue_invoice();
    }
}
