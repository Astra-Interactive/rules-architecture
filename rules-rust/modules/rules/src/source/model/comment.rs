use crate::source::model::TextPosition;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Comment {
    pub(crate) start: TextPosition,
    pub(crate) text: String,
    pub(crate) kind: CommentKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CommentKind {
    Line,
    Block,
}
