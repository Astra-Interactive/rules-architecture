#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Target {
    pub(crate) name: String,
    pub(crate) kinds: Vec<String>,
    pub(crate) src_path: String,
}
