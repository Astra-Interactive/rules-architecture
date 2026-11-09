use crate::invoice::api::InvoiceRepository;

pub struct FakeInvoiceRepository {
    count: usize,
}

impl FakeInvoiceRepository {
    pub fn new(count: usize) -> Self {
        Self { count }
    }
}

impl InvoiceRepository for FakeInvoiceRepository {
    fn count_invoices(&self) -> usize {
        self.count
    }
}
