use crate::source::model::NodeSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UnanalysableSource {
    pub(crate) site: UnanalysableSite,
    pub(crate) subject: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UnanalysableSite {
    Node { file: String, span: NodeSpan },
    Crate { path: String },
}
