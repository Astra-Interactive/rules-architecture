pub mod invoice {
    pub mod api {
        pub use invoice_repository::InvoiceRepository;
        mod invoice_repository;
    }
    pub mod model {
        pub use invoice_summary::InvoiceSummary;
        mod invoice_summary;
    }
    pub mod usecase {
        pub use count_invoices_use_case::CountInvoicesUseCase;
        mod count_invoices_use_case;
    }
    pub(crate) mod util {
        mod invoice_summary_ext;
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

#[cfg(test)]
#[path = "../test"]
mod test {
    mod invoice {
        mod usecase {
            mod count_invoices_use_case_test;
        }
        mod util {
            mod invoice_summary_ext_test;
        }
    }
}
