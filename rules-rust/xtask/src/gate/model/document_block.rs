#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DocumentBlock {
    pub(crate) document: String,
    pub(crate) heading: String,
    pub(crate) first_line: String,
    pub(crate) lines: Vec<String>,
}
