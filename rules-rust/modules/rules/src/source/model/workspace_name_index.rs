use crate::source::model::WorkspaceUse;

pub(crate) struct WorkspaceNameIndex {
    pub(crate) has_blind_spots: bool,
    pub(crate) uses: Vec<WorkspaceUse>,
    pub(crate) by_target: std::collections::BTreeMap<(usize, usize), Vec<usize>>,
}
