use crate::check::model::CheckError;
use crate::workspace::model::Workspace;
use std::path::Path;

pub(crate) trait WorkspaceReader: Send + Sync {
    fn read_workspace(&self, dir: &Path) -> Result<Workspace, CheckError>;
}
