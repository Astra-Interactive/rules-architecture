use crate::invoice::model::Invoice;

impl Invoice {
    pub(crate) fn is_numbered(&self) -> bool {
        self.number > 0
    }
}
