use crate::app::di::CoreModule;
use crate::feature_flag::service::IntervalFeatureFlagReloader;
use std::sync::Arc;
use std::time::Duration;

pub(crate) struct FeatureFlagModule {
    pub(crate) feature_flag_reloader: Arc<IntervalFeatureFlagReloader>,
}

impl FeatureFlagModule {
    pub(crate) fn new(core_module: &CoreModule) -> Self {
        let feature_flag_reloader = Arc::new(IntervalFeatureFlagReloader::new(
            Self::RELOAD_INTERVAL,
            Arc::clone(&core_module.task_scope),
        ));
        Self {
            feature_flag_reloader,
        }
    }

    const RELOAD_INTERVAL: Duration = Duration::from_secs(60);
}
