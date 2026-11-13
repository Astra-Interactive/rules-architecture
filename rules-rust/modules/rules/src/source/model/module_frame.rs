use crate::source::model::SourceTree;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModuleFrame {
    pub(crate) path: Vec<String>,
    pub(crate) file: String,
    pub(crate) directory: String,
    pub(crate) tree: SourceTree,
    pub(crate) is_root_file: bool,
}
