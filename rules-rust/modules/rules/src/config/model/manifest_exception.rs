#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ManifestException {
    pub(crate) scope: ManifestScope,
    pub(crate) rule_id: String,
    pub(crate) subject: String,
    pub(crate) reason: String,
    pub(crate) path: String,
    pub(crate) line: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ManifestScope {
    Package(String),
    Workspace,
}
