use crate::check::api::Rule;
use crate::ordering::rule::{CalleeBeforeCaller, DeclarationOrder};
use std::sync::Arc;

pub(crate) struct OrderingRuleModule {
    pub(crate) rules: Arc<[Arc<dyn Rule>]>,
    pub(crate) active_rule_ids: Vec<&'static str>,
}

impl OrderingRuleModule {
    pub(crate) fn new() -> Self {
        let rules: Vec<Arc<dyn Rule>> =
            vec![Arc::new(CalleeBeforeCaller), Arc::new(DeclarationOrder)];
        let active_rule_ids = vec!["CalleeBeforeCaller", "DeclarationOrder"];
        Self {
            rules: rules.into(),
            active_rule_ids,
        }
    }
}
