pub mod invoice {
    pub mod api {
        pub use invoice_repository::InvoiceRepository;
        mod invoice_repository;
    }
}

#[cfg(any(test, feature = "fake"))]
#[path = "../fixture"]
pub mod fixture {
    pub mod invoice {
        pub mod fake {
            pub use fake_invoice_repository::FakeInvoiceRepository;
            mod fake_invoice_repository;
        }
    }
}
