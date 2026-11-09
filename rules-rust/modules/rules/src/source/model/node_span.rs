use crate::source::model::TextPosition;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct NodeSpan {
    pub(crate) start: TextPosition,
    pub(crate) end: TextPosition,
}
