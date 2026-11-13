// modules/billing/api: invoice/file/file_invoice_repository.rs, used by InvoiceModule in the same crate
use crate::invoice::api::InvoiceRepository;
use crate::invoice::model::{Invoice, InvoiceError};
use std::path::PathBuf;

pub(crate) struct FileInvoiceRepository {
    folder: PathBuf,
}

impl FileInvoiceRepository {
    pub(crate) fn new(folder: PathBuf) -> Self {
        Self { folder }
    }
}

impl InvoiceRepository for FileInvoiceRepository {
    fn find_invoice(&self, number: u32) -> Result<Option<Invoice>, InvoiceError> {
        let path = self.folder.join(number.to_string());
        if !path.exists() {
            return Ok(None);
        }
        Ok(Some(Invoice { number }))
    }
}
