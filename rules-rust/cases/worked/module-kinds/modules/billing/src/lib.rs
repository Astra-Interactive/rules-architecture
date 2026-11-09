pub(crate) mod invoice {
    pub(crate) mod model {
        pub(crate) use invoice::{Invoice, InvoiceId};
        pub(crate) use invoice_error::InvoiceError;
        mod invoice;
        mod invoice_error;
    }
    pub(crate) mod api {
        pub(crate) use invoice_repository::InvoiceRepository;
        mod invoice_repository;
    }
    pub(crate) mod memory {
        pub(crate) use in_memory_invoice_repository::InMemoryInvoiceRepository;
        mod in_memory_invoice_repository;
    }
    pub(crate) mod usecase {
        pub(crate) use issue_invoice_use_case::IssueInvoiceUseCase;
        mod issue_invoice_use_case;
    }
    pub(crate) mod di {
        pub(crate) use invoice_module::InvoiceModule;
        mod invoice_module;
    }
}
pub(crate) mod order {
    pub(crate) mod usecase {
        pub(crate) use place_order_use_case::PlaceOrderUseCase;
        mod place_order_use_case;
    }
}

generated_items!();

#[cfg(test)]
#[path = "../test"]
mod test {
    mod invoice {
        mod usecase {
            mod issue_invoice_use_case_test;
        }
    }
}

