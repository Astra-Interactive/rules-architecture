use crate::source::model::ModuleId;
use crate::source::model::NodeSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Expectation {
    pub(crate) file: String,
    pub(crate) line: u32,
    pub(crate) position: ExpectationPosition,
    pub(crate) rule_ids: Vec<String>,
    pub(crate) reason: String,
    pub(crate) malformation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ExpectationPosition {
    Node(NodeSpan),
    ModuleDeclaration {
        module: ModuleId,
        declaration: NodeSpan,
    },
    ModuleInner(ModuleId),
    InlineModuleInner {
        module: ModuleId,
        declaration: NodeSpan,
    },
    File,
}
