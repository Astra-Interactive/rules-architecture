use crate::source::model::ModuleId;
use crate::source::model::NodeSpan;
use crate::source::model::SourceTree;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceModule {
    pub(crate) id: ModuleId,
    pub(crate) declaration: Option<ModuleDeclaration>,
    pub(crate) tree: SourceTree,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModuleDeclaration {
    pub(crate) file: String,
    pub(crate) body_file: String,
    pub(crate) span: NodeSpan,
    pub(crate) is_raw_identifier: bool,
    pub(crate) is_inline: bool,
    pub(crate) visibility: DeclaredVisibility,
    pub(crate) use_visibilities: Vec<DeclaredVisibility>,
    pub(crate) target_value: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeclaredVisibility {
    Private,
    Public,
    Crate,
    Restricted,
}
