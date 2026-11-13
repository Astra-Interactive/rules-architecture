use crate::source::model::CfgPredicate;
use std::collections::BTreeSet;

pub(crate) struct CycleComponent {
    pub(crate) modules: BTreeSet<usize>,
    pub(crate) edges: BTreeSet<usize>,
    pub(crate) cfg: CfgPredicate,
}
