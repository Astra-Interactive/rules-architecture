// task/api/scoped_job.rs, in the crate that declares TaskScope
use crate::task::model::TaskGuard;

pub trait ScopedJob: Send + Sync {
    fn start(&self) -> TaskGuard;
    fn reload(&self, running: &mut TaskGuard);
}
