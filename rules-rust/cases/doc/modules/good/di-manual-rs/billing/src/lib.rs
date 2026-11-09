pub mod invoice {
    pub(crate) mod api {
        pub(crate) use invoice_repository::InvoiceRepository;
        mod invoice_repository;
    }
    pub mod di {
        pub use invoice_module::InvoiceModule;
        mod invoice_module;
    }
    pub(crate) mod memory {
        pub(crate) use in_memory_invoice_repository::InMemoryInvoiceRepository;
        mod in_memory_invoice_repository;
    }
    pub(crate) mod usecase {
        pub(crate) use pay_invoice_use_case::PayInvoiceUseCase;
        mod pay_invoice_use_case;
    }
}
