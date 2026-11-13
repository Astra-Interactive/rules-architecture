use crate::check::api::Rule;
use crate::suppression::rule::ExpectScope;
use std::sync::Arc;

pub(crate) struct SuppressionRuleModule {
    pub(crate) rules: Arc<[Arc<dyn Rule>]>,
    pub(crate) active_rule_ids: Vec<&'static str>,
}

impl SuppressionRuleModule {
    pub(crate) fn new() -> Self {
        let rules: Vec<Arc<dyn Rule>> = vec![Arc::new(ExpectScope)];
        let active_rule_ids = vec!["ExpectScope"];
        Self {
            rules: rules.into(),
            active_rule_ids,
        }
    }
}
