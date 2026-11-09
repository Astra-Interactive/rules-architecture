use shop_billing_api::invoice::usecase::CountInvoicesUseCase;
use std::sync::Arc;

pub(crate) struct InvoiceCountCommand {
    count_invoices_use_case: Arc<CountInvoicesUseCase>,
}

impl InvoiceCountCommand {
    pub(crate) fn new(count_invoices_use_case: Arc<CountInvoicesUseCase>) -> Self {
        Self {
            count_invoices_use_case,
        }
    }

    pub(crate) fn format_reply(&self) -> String {
        format!("{} invoices", self.count_invoices_use_case.invoke())
    }
}
