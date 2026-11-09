use crate::fixture::invoice::fake::FakeInvoiceRepository;
use crate::invoice::usecase::CountInvoicesUseCase;
use std::sync::Arc;

#[test]
fn given_two_stored_invoices_when_counted_then_returns_two() {
    let count_invoices_use_case =
        CountInvoicesUseCase::new(Arc::new(FakeInvoiceRepository::new(2)));

    assert_eq!(2, count_invoices_use_case.invoke());
}
