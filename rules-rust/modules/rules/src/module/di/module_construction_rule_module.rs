use crate::check::api::Rule;
use crate::module::rule::{
    ConstructorDependency, DiModuleLifecycle, DiModuleShape, GlobalState, MainShape,
    RootModuleConstruction, SystemClockOutsideDi,
};
use std::sync::Arc;

pub(crate) struct ModuleConstructionRuleModule {
    pub(crate) rules: Arc<[Arc<dyn Rule>]>,
    pub(crate) active_rule_ids: Vec<&'static str>,
}

impl ModuleConstructionRuleModule {
    pub(crate) fn new() -> Self {
        let rules: Vec<Arc<dyn Rule>> = vec![
            Arc::new(GlobalState),
            Arc::new(DiModuleLifecycle),
            Arc::new(RootModuleConstruction),
            Arc::new(MainShape),
            Arc::new(SystemClockOutsideDi),
            Arc::new(DiModuleShape),
            Arc::new(ConstructorDependency),
        ];
        let active_rule_ids = vec![
            "GlobalState",
            "DiModuleLifecycle",
            "RootModuleConstruction",
            "MainShape",
            "SystemClockOutsideDi",
            "DiModuleShape",
            "ConstructorDependency",
        ];
        Self {
            rules: rules.into(),
            active_rule_ids,
        }
    }
}
