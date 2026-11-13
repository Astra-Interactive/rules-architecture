use crate::ledger::api::LedgerReader;
use case_billing_api::fixture::invoice::fake::FakeInvoiceRepository;
use case_billing_api::invoice::api::InvoiceRepository;

pub struct FakeLedgerReader {
    invoice_repository: FakeInvoiceRepository,
}

impl FakeLedgerReader {
    pub fn new(count: u32) -> Self {
        Self {
            invoice_repository: FakeInvoiceRepository::new(count),
        }
    }
}

impl LedgerReader for FakeLedgerReader {
    fn total(&self) -> u32 {
        self.invoice_repository.count()
    }
}
