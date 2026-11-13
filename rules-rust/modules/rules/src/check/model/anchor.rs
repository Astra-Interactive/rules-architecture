use crate::source::model::ModuleId;
use crate::source::model::NodeSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Anchor {
    Node {
        file: String,
        span: NodeSpan,
    },
    Module {
        module: ModuleId,
        file: String,
        body_file: String,
        declaration: NodeSpan,
    },
    File(String),
    Crate(String),
    Edge {
        from_crate: String,
        to_crate: String,
    },
    Workspace,
    Unexpectable,
}
