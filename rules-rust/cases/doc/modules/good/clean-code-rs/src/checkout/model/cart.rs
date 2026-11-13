use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Cart {
    pub(crate) expires_at: SystemTime,
}
