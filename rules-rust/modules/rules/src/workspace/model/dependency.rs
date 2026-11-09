#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Dependency {
    pub(crate) name: String,
    pub(crate) path: Option<String>,
    pub(crate) kind: DependencyKind,
    pub(crate) is_target_specific: bool,
    pub(crate) line: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DependencyKind {
    Normal,
    Development,
    Build,
}
