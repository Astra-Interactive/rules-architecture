use crate::fixture::invoice::fake::FakeInvoiceRepository;
use crate::invoice::usecase::CountInvoicesUseCase;
use std::sync::Arc;

#[test]
fn given_two_stored_invoices_when_counted_then_returns_two() {
    let invoice_repository = FakeInvoiceRepository::new(2);
    let count_invoices_use_case = CountInvoicesUseCase::new(Arc::new(invoice_repository));

    assert_eq!(2, count_invoices_use_case.invoke());
}

#[test]
fn given_active_and_archived_invoices_when_counted_then_excludes_archived_invoices() {
    let invoice_repository = FakeInvoiceRepository::with_archived(2, 3);
    let count_invoices_use_case = CountInvoicesUseCase::new(Arc::new(invoice_repository));

    assert_eq!(2, count_invoices_use_case.invoke());
}
