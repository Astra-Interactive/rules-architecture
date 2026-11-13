// invoice/model/invoice.rs
use crate::invoice::model::{InvoiceId, Money};
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Invoice {
    pub(crate) id: InvoiceId,
    pub(crate) total: Money,
    pub(crate) due_at: SystemTime,
    pub(crate) paid_at: Option<SystemTime>,
}
