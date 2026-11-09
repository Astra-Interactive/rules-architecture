use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GateArguments {
    pub(crate) docs: PathBuf,
    pub(crate) is_check: bool,
}
