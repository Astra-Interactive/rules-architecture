use crate::invoice::command::InvoiceCountCommand;
use shop_billing_api::fixture::invoice::fake::FakeInvoiceRepository;
use shop_billing_api::invoice::usecase::CountInvoicesUseCase;
use std::sync::Arc;

#[test]
fn given_three_invoices_when_reply_is_formatted_then_names_the_count() {
    let count_invoices_use_case = Arc::new(CountInvoicesUseCase::new(Arc::new(
        FakeInvoiceRepository::new(3),
    )));
    let invoice_count_command = InvoiceCountCommand::new(count_invoices_use_case);

    assert_eq!("3 invoices", invoice_count_command.format_reply());
}
