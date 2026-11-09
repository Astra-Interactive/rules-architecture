// invoice/util/invoice_ext.rs
use crate::invoice::model::Invoice;
use std::time::SystemTime;

impl Invoice {
    pub(crate) fn is_paid(&self) -> bool {
        self.paid_at.is_some()
    }

    pub(crate) fn is_overdue(&self, now: SystemTime) -> bool {
        !self.is_paid() && now > self.due_at
    }

    pub(crate) fn settle(self, now: SystemTime) -> Invoice {
        Invoice {
            paid_at: Some(now),
            ..self
        }
    }
}
