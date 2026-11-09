mod api { pub trait Repository {} }
mod di { struct InvoiceModule { repository: std::sync::Arc<dyn crate::api::Repository> } }
