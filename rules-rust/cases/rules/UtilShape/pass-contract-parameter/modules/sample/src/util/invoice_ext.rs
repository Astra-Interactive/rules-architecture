use crate::model::Invoice;
use std::time::SystemTime;

impl Invoice {
    pub(crate) fn settle(&self, moment: SystemTime) {}
}
