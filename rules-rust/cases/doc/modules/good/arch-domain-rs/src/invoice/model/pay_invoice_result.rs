use crate::invoice::model::{Invoice, InvoiceError};

#[derive(Debug)]
pub(crate) enum PayInvoiceResult {
    Paid(Invoice),
    AlreadyPaid,
    NotFound,
    Failed(InvoiceError),
}
