pub(crate) mod api {
    pub(crate) trait Repository {}
}
pub(crate) mod model {
    pub(crate) use invoice::Invoice;
    mod invoice;
}
pub(crate) mod util {
    mod invoice_ext;
    mod repository_scan;
}
