#[derive(Debug, thiserror::Error)]
pub enum InvoiceError {
    #[error("invoice {number} could not be read")]
    Unreadable { number: u32 },
}
