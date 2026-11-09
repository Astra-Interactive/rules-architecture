// invoice/api/invoice_repository.rs
use crate::invoice::model::{Invoice, InvoiceError, InvoiceId};
use async_trait::async_trait;

#[async_trait]
pub(crate) trait InvoiceRepository: Send + Sync {
    async fn find_invoice(&self, id: InvoiceId) -> Result<Option<Invoice>, InvoiceError>;
    async fn save_invoice(&self, invoice: Invoice) -> Result<(), InvoiceError>;
}
