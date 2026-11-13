use crate::ledger::usecase::CountLedgerUseCase;
use case_billing_api::fixture::invoice::fake::FakeInvoiceRepository;
use std::sync::Arc;

#[test]
fn given_two_invoices_when_counted_then_returns_two() {
    let count_ledger_use_case = CountLedgerUseCase::new(Arc::new(FakeInvoiceRepository::new(2)));

    assert_eq!(2, count_ledger_use_case.invoke());
}
