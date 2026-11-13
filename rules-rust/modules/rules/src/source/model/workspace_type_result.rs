use crate::source::model::{ExternalPath, WorkspaceResolvedPath};

pub(crate) struct WorkspaceTypeResult {
    pub(crate) items: Vec<WorkspaceResolvedPath>,
    pub(crate) external: Vec<ExternalPath>,
}
