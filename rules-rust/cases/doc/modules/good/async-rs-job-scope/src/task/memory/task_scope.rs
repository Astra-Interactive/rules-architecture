// task/memory/task_scope.rs, the job scope
use crate::task::api::ScopedJob;
use crate::task::model::TaskGuard;
use parking_lot::Mutex;
use std::sync::Arc;

pub struct TaskScope {
    running: Mutex<Vec<RunningJob>>,
}

impl TaskScope {
    pub fn new() -> Self {
        Self {
            running: Mutex::new(Vec::new()),
        }
    }

    fn take_guard(&self, job: &Arc<dyn ScopedJob>) -> Option<TaskGuard> {
        let mut running = self.running.lock();
        running
            .iter_mut()
            .find(|entry| Arc::ptr_eq(&entry.job, job))?
            .guard
            .take()
    }

    fn store_guard(&self, job: &Arc<dyn ScopedJob>, guard: TaskGuard) {
        let mut running = self.running.lock();
        let Some(entry) = running
            .iter_mut()
            .find(|entry| Arc::ptr_eq(&entry.job, job))
        else {
            return;
        };
        entry.guard = Some(guard);
    }

    pub fn spawn(&self, job: Arc<dyn ScopedJob>) {
        let guard = job.start();
        self.running.lock().push(RunningJob {
            job,
            guard: Some(guard),
        });
    }

    pub fn reload(&self) {
        let jobs: Vec<Arc<dyn ScopedJob>> = self
            .running
            .lock()
            .iter()
            .map(|running| Arc::clone(&running.job))
            .collect();
        for job in jobs {
            let Some(mut guard) = self.take_guard(&job) else {
                continue;
            };
            job.reload(&mut guard);
            self.store_guard(&job, guard);
        }
    }

    pub fn abort(&self) {
        let running = std::mem::take(&mut *self.running.lock());
        drop(running);
    }
}

impl Default for TaskScope {
    fn default() -> Self {
        Self::new()
    }
}

struct RunningJob {
    job: Arc<dyn ScopedJob>,
    guard: Option<TaskGuard>,
}
