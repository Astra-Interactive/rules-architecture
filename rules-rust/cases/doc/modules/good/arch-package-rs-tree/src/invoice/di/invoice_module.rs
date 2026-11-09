use crate::invoice::file::FileInvoiceRepository;
use crate::invoice::usecase::IssueInvoiceUseCase;
use std::path::PathBuf;
use std::sync::Arc;

pub struct InvoiceModule {
    pub issue_invoice_use_case: Arc<IssueInvoiceUseCase>,
}

impl InvoiceModule {
    pub fn new(folder: PathBuf) -> Self {
        let invoice_repository = Arc::new(FileInvoiceRepository::new(folder));
        Self {
            issue_invoice_use_case: Arc::new(IssueInvoiceUseCase::new(invoice_repository)),
        }
    }
}
