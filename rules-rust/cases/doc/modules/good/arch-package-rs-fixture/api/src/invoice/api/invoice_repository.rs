pub trait InvoiceRepository: Send + Sync {
    fn count_invoices(&self) -> usize;
}
