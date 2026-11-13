use case_billing_api::fixture::invoice::fake::FakeInvoiceRepository;
use case_ledger::ledger::usecase::CountLedgerUseCase;
use std::sync::Arc;

#[test]
fn given_three_invoices_when_counted_through_the_public_api_then_returns_three() {
    let count_ledger_use_case = CountLedgerUseCase::new(Arc::new(FakeInvoiceRepository::new(3)));

    assert_eq!(3, count_ledger_use_case.invoke());
}
