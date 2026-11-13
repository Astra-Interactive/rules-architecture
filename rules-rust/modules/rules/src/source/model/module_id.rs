#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct ModuleId {
    pub(crate) crate_name: String,
    pub(crate) target: String,
    pub(crate) path: Vec<String>,
}
