use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct LifecycleFacts {
    pub(crate) owned: Vec<String>,
    pub(crate) scopes: Vec<String>,
    pub(crate) aborted: BTreeSet<String>,
    pub(crate) invalid: bool,
    pub(crate) needs_lifecycle: bool,
}
