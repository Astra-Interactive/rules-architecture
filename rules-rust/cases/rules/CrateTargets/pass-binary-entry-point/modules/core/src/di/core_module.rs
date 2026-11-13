use crate::lifecycle::api::Lifecycle;
use std::path::PathBuf;

pub struct CoreModule {
    config_path: PathBuf,
}

impl CoreModule {
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }
}

impl Lifecycle for CoreModule {
    fn on_enable(&self) {
        tracing::info!(config = %self.config_path.display(), "Core enabled");
    }

    fn on_disable(&self) {
        tracing::info!("Core disabled");
    }
}
