use std::time::SystemTime;

pub(crate) trait Clock: Send + Sync {
    fn now(&self) -> SystemTime;
}
