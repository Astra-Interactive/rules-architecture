#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExternalPath {
    pub(crate) package: String,
    pub(crate) segments: Vec<String>,
    pub(crate) member: Option<usize>,
}
