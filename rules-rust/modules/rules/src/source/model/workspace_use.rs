use crate::source::model::{CfgPredicate, NodeSpan};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WorkspaceUseKind {
    Import,
    Type,
    Call,
    Construction,
    Value,
    TraitImplementation,
}

#[derive(Clone)]
pub(crate) struct WorkspaceUse {
    pub(crate) source_index: usize,
    pub(crate) source_module: usize,
    pub(crate) file: String,
    pub(crate) span: NodeSpan,
    pub(crate) anchor: NodeSpan,
    pub(crate) cfg: CfgPredicate,
    pub(crate) target_index: usize,
    pub(crate) target_item: usize,
    pub(crate) named_targets: Vec<crate::source::model::WorkspaceResolvedPath>,
    pub(crate) kind: WorkspaceUseKind,
    pub(crate) function: Option<String>,
    pub(crate) block: Option<NodeSpan>,
    pub(crate) is_test: bool,
    pub(crate) ambiguous: bool,
    pub(crate) path: Vec<String>,
}
