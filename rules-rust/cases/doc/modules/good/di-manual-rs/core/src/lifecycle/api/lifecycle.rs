pub trait Lifecycle {
    fn on_enable(&self);

    fn on_disable(&self);

    fn on_reload(&self) {}
}
