#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct TextPosition {
    pub(crate) line: u32,
    pub(crate) column: u32,
}
