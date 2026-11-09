// instances/server: di/root_module.rs
use di_billing_api::invoice::di::InvoiceModule;
use shop_core::di::CoreModule;
use shop_core::lifecycle::api::Lifecycle;
use std::path::PathBuf;

pub(crate) struct RootModule {
    core_module: CoreModule,
    invoice_module: InvoiceModule,
}

impl RootModule {
    pub(crate) fn new(config_path: PathBuf) -> Self {
        let core_module = CoreModule::new(config_path);
        let invoice_module = InvoiceModule::new(&core_module);
        Self {
            core_module,
            invoice_module,
        }
    }
}

impl Lifecycle for RootModule {
    fn on_enable(&self) {
        self.core_module.on_enable();
        self.invoice_module.on_enable();
    }

    fn on_disable(&self) {
        self.invoice_module.on_disable();
        self.core_module.on_disable();
    }
}
