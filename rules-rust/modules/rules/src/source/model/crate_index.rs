use crate::source::model::CfgPredicate;
use crate::source::model::ModuleId;
use crate::source::model::NodeSpan;
use crate::source::model::SourceReference;
use crate::source::model::SourceTree;
use crate::source::model::UnanalysableSource;

#[derive(Clone)]
pub(crate) struct CrateIndex {
    pub(crate) crate_name: String,
    pub(crate) target: String,
    pub(crate) dependencies: Vec<String>,
    pub(crate) modules: Vec<IndexedModule>,
    pub(crate) items: Vec<IndexedItem>,
    pub(crate) implementations: Vec<IndexedImpl>,
    pub(crate) imports: Vec<ImportBinding>,
    pub(crate) edges: Vec<SourceEdge>,
    pub(crate) unanalysable: Vec<UnanalysableSource>,
    pub(crate) references: Vec<SourceReference>,
}

#[derive(Clone)]
pub(crate) struct IndexedModule {
    pub(crate) id: usize,
    pub(crate) parent: Option<usize>,
    pub(crate) module: ModuleId,
    pub(crate) file: String,
    pub(crate) span: NodeSpan,
    pub(crate) tree: SourceTree,
    pub(crate) cfg: CfgPredicate,
    pub(crate) is_file_module: bool,
}

#[derive(Clone)]
pub(crate) struct IndexedItem {
    pub(crate) module: usize,
    pub(crate) file: String,
    pub(crate) span: NodeSpan,
    pub(crate) name: String,
    pub(crate) cfg: CfgPredicate,
    pub(crate) syntax: syn::Item,
}

#[derive(Clone)]
pub(crate) struct IndexedImpl {
    pub(crate) module: usize,
    pub(crate) file: String,
    pub(crate) span: NodeSpan,
    pub(crate) cfg: CfgPredicate,
    pub(crate) syntax: syn::ItemImpl,
}

#[derive(Clone)]
pub(crate) struct ImportBinding {
    pub(crate) module: usize,
    pub(crate) file: String,
    pub(crate) span: NodeSpan,
    pub(crate) name: Option<String>,
    pub(crate) path: Vec<String>,
    pub(crate) is_glob: bool,
    pub(crate) is_renamed: bool,
    pub(crate) is_public: bool,
    pub(crate) is_externally_public: bool,
    pub(crate) is_crate_public: bool,
    pub(crate) cfg: CfgPredicate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Resolution {
    Internal(Vec<ResolvedPath>),
    External(String),
    Local,
    ExternalGlob,
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedPath {
    pub(crate) module: usize,
    pub(crate) item: Option<usize>,
    pub(crate) cfg: CfgPredicate,
}

#[derive(Clone)]
pub(crate) struct SourceEdge {
    pub(crate) source: usize,
    pub(crate) target: usize,
    pub(crate) item: Option<usize>,
    pub(crate) file: String,
    pub(crate) span: NodeSpan,
    pub(crate) cfg: CfgPredicate,
    pub(crate) is_test: bool,
    pub(crate) path: Vec<String>,
}
