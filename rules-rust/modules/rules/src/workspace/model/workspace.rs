use crate::workspace::model::Member;
use crate::workspace::model::TomlFile;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Workspace {
    pub(crate) root: PathBuf,
    pub(crate) target_directory: PathBuf,
    pub(crate) root_manifest: TomlFile,
    pub(crate) members: Vec<Member>,
    pub(crate) clippy_files: Vec<TomlFile>,
    pub(crate) rustfmt_files: Vec<TomlFile>,
    pub(crate) toolchain_file: Option<ToolchainFile>,
    pub(crate) cargo_config: Option<TomlFile>,
    pub(crate) files: Vec<String>,
    pub(crate) directories: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ToolchainFile {
    Toml(TomlFile),
    Legacy { path: String, channel: String },
}
