pub trait InvoiceRepository: Send + Sync {
    fn count(&self) -> u32;
}
