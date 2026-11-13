use crate::source::model::{NodeSpan, WorkspaceUseKind};

pub(crate) struct ReferenceSite {
    pub(crate) span: NodeSpan,
    pub(crate) kind: WorkspaceUseKind,
    pub(crate) function: Option<String>,
    pub(crate) block: Option<NodeSpan>,
    pub(crate) path: Vec<String>,
    pub(crate) syntax: syn::Path,
}
