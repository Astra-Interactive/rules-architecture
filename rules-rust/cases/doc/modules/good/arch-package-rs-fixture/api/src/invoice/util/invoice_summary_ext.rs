use crate::invoice::model::InvoiceSummary;

impl InvoiceSummary {
    pub(crate) fn is_countable(&self) -> bool {
        matches!(self, Self::Active)
    }
}
