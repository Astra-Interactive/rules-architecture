use crate::check::api::Rule;
use crate::layout::rule::{
    ContainerSources, CrateTargets, NestedUnit, PlatformSourceSetUnit, PluginImplUnit,
    ProjectDirMatchesPath, PublishedVersion, RootBuildSubprojects, RootPackagePath, RootSources,
    ShellAutomation, UnitPath, XtaskAlias,
};
use std::sync::Arc;

pub(crate) struct LayoutRuleModule {
    pub(crate) rules: Arc<[Arc<dyn Rule>]>,
    pub(crate) active_rule_ids: Vec<&'static str>,
}

impl LayoutRuleModule {
    pub(crate) fn new() -> Self {
        let rules: Vec<Arc<dyn Rule>> = vec![
            Arc::new(ContainerSources),
            Arc::new(CrateTargets),
            Arc::new(NestedUnit),
            Arc::new(ProjectDirMatchesPath),
            Arc::new(RootBuildSubprojects),
            Arc::new(RootSources),
            Arc::new(UnitPath),
            Arc::new(RootPackagePath),
            Arc::new(PublishedVersion),
            Arc::new(XtaskAlias),
            Arc::new(ShellAutomation),
            Arc::new(PlatformSourceSetUnit),
            Arc::new(PluginImplUnit),
        ];
        let active_rule_ids = vec![
            "ContainerSources",
            "CrateTargets",
            "NestedUnit",
            "ProjectDirMatchesPath",
            "RootBuildSubprojects",
            "RootSources",
            "UnitPath",
            "RootPackagePath",
            "PublishedVersion",
            "XtaskAlias",
            "ShellAutomation",
            "PlatformSourceSetUnit",
            "PluginImplUnit",
        ];
        Self {
            rules: rules.into(),
            active_rule_ids,
        }
    }
}
