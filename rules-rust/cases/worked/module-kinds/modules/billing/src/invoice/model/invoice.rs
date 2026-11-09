use crate::invoice::api::InvoiceRepository;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Invoice {
    pub(crate) id: InvoiceId,
    pub(crate) total: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct InvoiceId(pub(crate) u64);

