use shop_core::di::CoreModule;
use shop_core::lifecycle::api::Lifecycle;
use std::path::PathBuf;

pub struct RootModule {
    core_module: CoreModule,
}

impl RootModule {
    pub fn new(config_path: PathBuf) -> Self {
        let core_module = CoreModule::new(config_path);
        Self { core_module }
    }
}

impl Lifecycle for RootModule {
    fn on_enable(&self) {
        self.core_module.on_enable();
    }

    fn on_disable(&self) {
        self.core_module.on_disable();
    }
}
