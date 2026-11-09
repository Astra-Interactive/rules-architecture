use crate::app::service::TaskScope;
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
