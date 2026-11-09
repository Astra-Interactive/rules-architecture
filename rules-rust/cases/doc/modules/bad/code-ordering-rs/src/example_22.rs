use std::sync::Arc;
use std::time::Duration;

pub(crate) struct TaskScope;

pub(crate) struct CoreModule {
    pub(crate) task_scope: Arc<TaskScope>,
}

pub(crate) struct IntervalFeatureFlagReloader {
    interval: Duration,
    task_scope: Arc<TaskScope>,
}

impl IntervalFeatureFlagReloader {
    pub(crate) fn new(interval: Duration, task_scope: Arc<TaskScope>) -> Self {
        Self {
            interval,
            task_scope,
        }
    }
}

struct FeatureFlagModuleDefaults;

impl FeatureFlagModuleDefaults {
    const RELOAD_INTERVAL: Duration = Duration::from_secs(60);
}

pub(crate) struct FeatureFlagModule {
    pub(crate) feature_flag_reloader: Arc<IntervalFeatureFlagReloader>,
}

impl FeatureFlagModule {
    pub(crate) fn new(core_module: &CoreModule) -> Self {
        let feature_flag_reloader = Arc::new(IntervalFeatureFlagReloader::new(
            FeatureFlagModuleDefaults::RELOAD_INTERVAL,
            Arc::clone(&core_module.task_scope),
        ));
        Self {
            feature_flag_reloader,
        }
    }
}
