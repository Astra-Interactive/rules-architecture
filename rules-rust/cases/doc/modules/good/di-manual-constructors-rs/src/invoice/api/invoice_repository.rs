pub(crate) trait InvoiceRepository: Send + Sync {
    fn count_unpaid(&self) -> usize;
}
