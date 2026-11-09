use crate::clock::api::Clock;
use crate::clock::system::SystemClock;
use crate::lifecycle::api::Lifecycle;
use std::path::PathBuf;
use std::sync::Arc;

pub struct CoreModule {
    pub clock: Arc<dyn Clock>,
    config_path: PathBuf,
}

impl CoreModule {
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            clock: Arc::new(SystemClock),
            config_path,
        }
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
