use crate::invoice::api::InvoiceRepository;
use crate::invoice::model::InvoiceSummary;

pub struct FakeInvoiceRepository {
    count: usize,
    archived: usize,
}

impl FakeInvoiceRepository {
    pub fn new(count: usize) -> Self {
        Self { count, archived: 0 }
    }

    pub fn with_archived(count: usize, archived: usize) -> Self {
        Self { count, archived }
    }
}

impl InvoiceRepository for FakeInvoiceRepository {
    fn invoices(&self) -> Vec<InvoiceSummary> {
        (0..self.count)
            .map(|_| InvoiceSummary::Active)
            .chain((0..self.archived).map(|_| InvoiceSummary::Archived))
            .collect()
    }
}
