// reminder/service/task_scope.rs, beside its only user, reminder/service/invoice_reminder.rs
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use tokio::sync::mpsc;
use tokio::task::{JoinHandle, JoinSet};

type Task = Pin<Box<dyn Future<Output = ()> + Send>>;

pub(crate) struct TaskScope {
    tasks: mpsc::UnboundedSender<Task>,
    drain: JoinHandle<()>,
}

impl TaskScope {
    pub(crate) fn new() -> Self {
        let (tasks, receiver) = mpsc::unbounded_channel();
        let drain = tokio::spawn(Self::drain(receiver));
        Self { tasks, drain }
    }

    async fn drain(mut receiver: mpsc::UnboundedReceiver<Task>) {
        let mut running = JoinSet::new();
        loop {
            tokio::select! {
                task = receiver.recv() => {
                    let Some(task) = task else {
                        return;
                    };
                    running.spawn(task);
                }
                Some(joined) = running.join_next() => {
                    let Err(error) = joined else {
                        continue;
                    };
                    if error.is_panic() {
                        tracing::error!(error = &error as &dyn Error, "A task panicked");
                    }
                }
            }
        }
    }

    pub(crate) fn spawn(&self, task: impl Future<Output = ()> + Send + 'static) {
        #[expect(
            clippy::let_underscore_must_use,
            reason = "the drain is gone only after abort, and a task spawned then is refused on purpose"
        )]
        let _ = self.tasks.send(Box::pin(task));
    }

    pub(crate) fn abort(&self) {
        self.drain.abort();
    }
}
