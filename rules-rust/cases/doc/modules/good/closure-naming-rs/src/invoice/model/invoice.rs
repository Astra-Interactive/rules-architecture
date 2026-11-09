use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Invoice {
    pub(crate) due_at: SystemTime,
    pub(crate) is_paid: bool,
}
