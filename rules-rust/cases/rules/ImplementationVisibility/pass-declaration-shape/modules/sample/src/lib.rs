pub(crate) mod api {
    pub(crate) use invoice_repository::InvoiceRepository;
    mod invoice_repository;
}
pub(crate) mod memory {
    mod in_memory_invoice_repository;
}
