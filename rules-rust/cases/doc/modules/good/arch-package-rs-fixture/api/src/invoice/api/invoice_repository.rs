use crate::invoice::model::InvoiceSummary;

pub trait InvoiceRepository: Send + Sync {
    fn invoices(&self) -> Vec<InvoiceSummary>;
}
