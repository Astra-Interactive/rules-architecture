pub(crate) mod invoice {
    pub(crate) mod model {
        pub(crate) use invoice::Invoice;
        mod invoice;
    }
    pub(crate) mod util {
        pub(crate) use invoice_report::write_invoice_report;
        mod invoice_report;
    }
}
