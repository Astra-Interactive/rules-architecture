pub(crate) trait InvoiceRepository: Send + Sync {
    fn mark_paid(&self, number: u32) -> bool;
}
