use crate::invoice::model::InvoiceId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Invoice {
    pub(crate) id: InvoiceId,
    pub(crate) total_cents: i64,
}
