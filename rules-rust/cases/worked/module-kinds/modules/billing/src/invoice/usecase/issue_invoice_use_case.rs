use crate::invoice::api::InvoiceRepository;
use crate::invoice::di::InvoiceModule;
use crate::order::usecase::PlaceOrderUseCase;
use std::sync::Arc;

pub(crate) struct IssueInvoiceUseCase {
    invoice_repository: Arc<dyn InvoiceRepository>,
}

impl IssueInvoiceUseCase {
    pub(crate) fn new(
        _other: &PlaceOrderUseCase,
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
