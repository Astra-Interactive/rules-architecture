pub(crate) trait Lifecycle: Send + Sync {
    fn on_enable(&self);
    fn on_disable(&self);
}
