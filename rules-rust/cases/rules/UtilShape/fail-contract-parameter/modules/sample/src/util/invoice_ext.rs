use crate::api::Repository;
use crate::model::Invoice;

impl Invoice {
    pub(crate) fn settle(&self, repository: &dyn Repository) {}
}
