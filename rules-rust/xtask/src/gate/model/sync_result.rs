use crate::gate::model::SyncChange;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SyncResult {
    pub(crate) changes: Vec<SyncChange>,
    pub(crate) hashes: Vec<String>,
}
