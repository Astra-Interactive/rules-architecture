mod api { pub trait Repository {} }
mod di { struct InvoiceModule<T: crate::api::Repository>(T); }
