use crate::source::model::ResolvedPath;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkspaceResolvedPath {
    pub(crate) index: usize,
    pub(crate) path: ResolvedPath,
}
