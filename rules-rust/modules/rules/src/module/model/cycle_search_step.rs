use crate::source::model::CfgPredicate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CycleSearchStep {
    pub(crate) module: usize,
    pub(crate) cfg: CfgPredicate,
    pub(crate) edges: Vec<usize>,
}
