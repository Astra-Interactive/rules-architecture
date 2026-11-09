pub(crate) trait InvoiceRepository: Send + Sync {
    fn can_issue_invoice(&self) -> bool;
    fn issue_invoice(&self);
}

