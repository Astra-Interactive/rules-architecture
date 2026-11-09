use crate::check::api::Rule;
use crate::default::rule::StructDefaultValue;
use std::sync::Arc;

pub(crate) struct DefaultRuleModule {
    pub(crate) rules: Arc<[Arc<dyn Rule>]>,
    pub(crate) active_rule_ids: Vec<&'static str>,
}

impl DefaultRuleModule {
    pub(crate) fn new() -> Self {
        let rules: Vec<Arc<dyn Rule>> = vec![Arc::new(StructDefaultValue)];
        let active_rule_ids = vec!["StructDefaultValue"];
        Self {
            rules: rules.into(),
            active_rule_ids,
        }
    }
}
