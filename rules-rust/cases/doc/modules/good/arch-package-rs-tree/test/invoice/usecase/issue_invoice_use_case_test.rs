use crate::invoice::model::Invoice;
use crate::invoice::usecase::IssueInvoiceUseCase;
use crate::test::invoice::fake::FakeInvoiceRepository;
use std::sync::Arc;

#[test]
fn given_numbered_invoice_when_issued_then_returns_it() {
    let invoice = Invoice { number: 7 };
    let fake_invoice_repository = Arc::new(FakeInvoiceRepository::new(Some(invoice.clone())));
    let issue_invoice_use_case = IssueInvoiceUseCase::new(fake_invoice_repository);

    let issued = issue_invoice_use_case
        .invoke(7)
        .expect("the fake never fails");

    assert_eq!(Some(invoice), issued);
}
