use crate::app::service::TaskScope;
use crate::lifecycle::api::Lifecycle;
use std::sync::Arc;

pub(crate) struct CoreModule {
    pub(crate) task_scope: Arc<TaskScope>,
}

impl CoreModule {
    pub(crate) fn new() -> Self {
        Self {
            task_scope: Arc::new(TaskScope::new("core".to_owned())),
        }
    }
}

impl Lifecycle for CoreModule {
    fn on_enable(&self) {}

    fn on_disable(&self) {
        self.task_scope.abort();
    }
}
