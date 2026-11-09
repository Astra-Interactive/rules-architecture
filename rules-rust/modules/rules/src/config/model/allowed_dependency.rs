#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AllowedDependency {
    pub(crate) from_crate: String,
    pub(crate) to_crate: String,
    pub(crate) reason: String,
    pub(crate) path: String,
    pub(crate) line: u32,
}
