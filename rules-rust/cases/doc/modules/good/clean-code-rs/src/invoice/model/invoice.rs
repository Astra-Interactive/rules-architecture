use crate::invoice::model::InvoiceId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Invoice {
    pub(crate) id: InvoiceId,
}
