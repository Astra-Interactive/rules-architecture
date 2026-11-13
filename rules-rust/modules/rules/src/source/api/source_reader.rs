use crate::source::model::CrateSources;
use crate::workspace::model::Workspace;

pub(crate) trait SourceReader: Send + Sync {
    fn read_sources(&self, workspace: &Workspace) -> Vec<CrateSources>;
}
