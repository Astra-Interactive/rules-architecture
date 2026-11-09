use crate::invoice::model::InvoiceSummary;

#[test]
fn given_active_and_archived_invoices_when_countability_is_checked_then_only_active_is_countable() {
    assert!(InvoiceSummary::Active.is_countable());
    assert!(!InvoiceSummary::Archived.is_countable());
}
