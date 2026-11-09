use crate::check::api::Rule;
use crate::tuple::rule::AnonymousTuple;
use std::sync::Arc;

pub(crate) struct TupleRuleModule {
    pub(crate) rules: Arc<[Arc<dyn Rule>]>,
    pub(crate) active_rule_ids: Vec<&'static str>,
}

impl TupleRuleModule {
    pub(crate) fn new() -> Self {
        let rules: Vec<Arc<dyn Rule>> = vec![Arc::new(AnonymousTuple)];
        let active_rule_ids = vec!["AnonymousTuple"];
        Self {
            rules: rules.into(),
            active_rule_ids,
        }
    }
}
