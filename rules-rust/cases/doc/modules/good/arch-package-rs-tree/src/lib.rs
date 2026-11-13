// modules/billing/api/src/lib.rs
pub mod invoice {
    pub mod api {
        pub use invoice_repository::InvoiceRepository;
        mod invoice_repository;
    }
    pub mod model {
        pub use invoice::Invoice;
        pub use invoice_error::InvoiceError;
        mod invoice;
        mod invoice_error;
    }
    pub mod usecase {
        pub use issue_invoice_use_case::IssueInvoiceUseCase;
        mod issue_invoice_use_case;
    }
    pub(crate) mod util {
        mod invoice_ext;
    }
    pub(crate) mod file {
        pub(crate) use file_invoice_repository::FileInvoiceRepository;
        mod file_invoice_repository;
    }
    pub mod di {
        pub use invoice_module::InvoiceModule;
        mod invoice_module;
    }
}

#[cfg(test)]
#[path = "../test"]
mod test {
    mod invoice {
        pub(crate) mod fake {
            pub(crate) use fake_invoice_repository::FakeInvoiceRepository;
            mod fake_invoice_repository;
        }
        mod usecase {
            mod issue_invoice_use_case_test;
        }
    }
}
