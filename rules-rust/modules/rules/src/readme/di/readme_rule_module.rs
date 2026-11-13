use crate::check::api::Rule;
use crate::readme::rule::UnitReadme;
use std::sync::Arc;

pub(crate) struct ReadmeRuleModule {
    pub(crate) rules: Arc<[Arc<dyn Rule>]>,
    pub(crate) active_rule_ids: Vec<&'static str>,
}

impl ReadmeRuleModule {
    pub(crate) fn new() -> Self {
        let rules: Vec<Arc<dyn Rule>> = vec![Arc::new(UnitReadme)];
        let active_rule_ids = vec!["UnitReadme"];
        Self {
            rules: rules.into(),
            active_rule_ids,
        }
    }
}
