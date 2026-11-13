use crate::source::model::ExpectationPosition;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExpectationSlot {
    pub(crate) rule_id: String,
    pub(crate) file: String,
    pub(crate) line: u32,
    pub(crate) position: ExpectationPosition,
    pub(crate) reason: String,
}
