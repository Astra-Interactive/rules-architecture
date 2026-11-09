use crate::config::model::AllowedDependency;
use crate::config::model::CrateConfig;
use crate::config::model::ManifestException;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProjectConfig {
    pub(crate) platform_words: Vec<String>,
    pub(crate) crates: Vec<CrateConfig>,
    pub(crate) manifest_exceptions: Vec<ManifestException>,
    pub(crate) allowed_dependencies: Vec<AllowedDependency>,
}
