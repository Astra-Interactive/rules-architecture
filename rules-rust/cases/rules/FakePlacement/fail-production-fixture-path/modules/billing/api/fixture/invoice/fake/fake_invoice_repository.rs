use crate::invoice::api::InvoiceRepository;

pub struct FakeInvoiceRepository {
    count: u32,
}

impl FakeInvoiceRepository {
    pub fn new(count: u32) -> Self {
        Self { count }
    }
}

impl InvoiceRepository for FakeInvoiceRepository {
    fn count(&self) -> u32 {
        self.count
    }
}
