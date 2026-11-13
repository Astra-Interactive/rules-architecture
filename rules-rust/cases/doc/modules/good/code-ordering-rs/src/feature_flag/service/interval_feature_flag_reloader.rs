use crate::app::service::TaskScope;
use std::sync::Arc;
use std::time::Duration;

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
