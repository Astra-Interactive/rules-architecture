pub(crate) trait OrderRepository: Send + Sync {
    fn count_orders(&self) -> usize;
}
