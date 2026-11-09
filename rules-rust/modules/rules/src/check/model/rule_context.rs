use crate::config::model::ProjectConfig;
use crate::source::model::CrateSources;
use crate::workspace::model::RustfmtFacts;
use crate::workspace::model::Workspace;

pub(crate) struct RuleContext {
    pub(crate) workspace: Workspace,
    pub(crate) project_config: ProjectConfig,
    pub(crate) crates: Vec<CrateSources>,
    pub(crate) rustfmt: RustfmtFacts,
}
