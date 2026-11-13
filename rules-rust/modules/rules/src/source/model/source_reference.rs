use crate::source::model::{CfgPredicate, NodeSpan, Resolution};

#[derive(Clone)]
pub(crate) struct SourceReference {
    pub(crate) module: usize,
    pub(crate) file: String,
    pub(crate) span: NodeSpan,
    pub(crate) anchor: NodeSpan,
    pub(crate) resolution: Resolution,
    pub(crate) cfg: CfgPredicate,
}
