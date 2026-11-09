// invoice/usecase/pay_invoice_use_case.rs
use crate::clock::api::Clock;
use crate::invoice::api::InvoiceRepository;
use crate::invoice::model::{InvoiceId, PayInvoiceResult};
use std::sync::Arc;

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

    pub(crate) async fn invoke(&self, id: InvoiceId) -> PayInvoiceResult {
        let invoice = match self.invoice_repository.find_invoice(id).await {
            Ok(Some(invoice)) => invoice,
            Ok(None) => return PayInvoiceResult::NotFound,
            Err(error) => return PayInvoiceResult::Failed(error),
        };
        if invoice.is_paid() {
            return PayInvoiceResult::AlreadyPaid;
        }
        let paid = invoice.settle(self.clock.now());
        match self.invoice_repository.save_invoice(paid.clone()).await {
            Ok(()) => PayInvoiceResult::Paid(paid),
            Err(error) => PayInvoiceResult::Failed(error),
        }
    }
}
