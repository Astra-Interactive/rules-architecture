use crate::module::model::ModuleIdentity;
use crate::source::model::CfgPredicate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModuleDependencyPath {
    pub(crate) identity: ModuleIdentity,
    pub(crate) cfg: CfgPredicate,
    pub(crate) visited: Vec<ModuleIdentity>,
}
