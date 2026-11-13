#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiredConfiguration {
    pub workspace_lints: String,
    pub clippy_toml: String,
}
