use parking_lot::Mutex;
use std::future::Future;
use tokio::task::JoinHandle;

pub(crate) struct TaskScope {
    tasks: Mutex<Vec<JoinHandle<()>>>,
}

impl TaskScope {
    pub(crate) fn new() -> Self {
        Self {
            tasks: Mutex::new(Vec::new()),
        }
    }

    pub(crate) fn spawn(&self, task: impl Future<Output = ()> + Send + 'static) {
        let handle = tokio::spawn(task);
        self.tasks.lock().push(handle);
    }
}
