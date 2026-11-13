use crate::invoice::model::{Invoice, InvoiceError};

pub trait InvoiceRepository: Send + Sync {
    fn find_invoice(&self, number: u32) -> Result<Option<Invoice>, InvoiceError>;
}
