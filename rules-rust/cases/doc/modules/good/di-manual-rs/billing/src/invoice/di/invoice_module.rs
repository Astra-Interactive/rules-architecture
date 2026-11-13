// modules/billing/api: invoice/di/invoice_module.rs, public because the entry-point crate constructs it
use crate::invoice::api::InvoiceRepository;
use crate::invoice::memory::InMemoryInvoiceRepository;
use crate::invoice::usecase::PayInvoiceUseCase;
use shop_core::di::CoreModule;
use shop_core::lifecycle::api::Lifecycle;
use std::sync::Arc;

pub struct InvoiceModule {
    pub(crate) invoice_repository: Arc<dyn InvoiceRepository>,
    pub(crate) pay_invoice_use_case: Arc<PayInvoiceUseCase>,
}

impl InvoiceModule {
    pub fn new(core_module: &CoreModule) -> Self {
        let in_memory_invoice_repository = InMemoryInvoiceRepository::new();
        let invoice_repository: Arc<dyn InvoiceRepository> = Arc::new(in_memory_invoice_repository);
        let pay_invoice_use_case = Arc::new(PayInvoiceUseCase::new(
            Arc::clone(&invoice_repository),
            Arc::clone(&core_module.clock),
        ));
        Self {
            invoice_repository,
            pay_invoice_use_case,
        }
    }
}

impl Lifecycle for InvoiceModule {
    fn on_enable(&self) {
        tracing::info!("Invoices enabled");
    }

    fn on_disable(&self) {
        tracing::info!("Invoices disabled");
    }
}
