use crate::check::api::Rule;
use crate::dependency::rule::{
    ApiUnitDependency, ApiUnitPlatformDependency, DependencyFloor, EntryPointDependency,
    PlatformDependencyTable, TechnologyDependency, TimeCrate, UnitDependency, UnitDependencyCycle,
};
use std::sync::Arc;

pub(crate) struct DependencyRuleModule {
    pub(crate) rules: Arc<[Arc<dyn Rule>]>,
    pub(crate) active_rule_ids: Vec<&'static str>,
}

impl DependencyRuleModule {
    pub(crate) fn new() -> Self {
        let rules: Vec<Arc<dyn Rule>> = vec![
            Arc::new(EntryPointDependency),
            Arc::new(UnitDependencyCycle),
            Arc::new(DependencyFloor),
            Arc::new(TimeCrate),
            Arc::new(UnitDependency),
            Arc::new(ApiUnitDependency),
            Arc::new(TechnologyDependency),
            Arc::new(ApiUnitPlatformDependency),
            Arc::new(PlatformDependencyTable),
        ];
        let active_rule_ids = vec![
            "EntryPointDependency",
            "UnitDependencyCycle",
            "DependencyFloor",
            "TimeCrate",
            "UnitDependency",
            "ApiUnitDependency",
            "TechnologyDependency",
            "ApiUnitPlatformDependency",
            "PlatformDependencyTable",
        ];
        Self {
            rules: rules.into(),
            active_rule_ids,
        }
    }
}
