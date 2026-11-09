use crate::check::api::Rule;
use crate::import::rule::{ImportOrdering, WildcardImport};
use std::sync::Arc;

pub(crate) struct ImportRuleModule {
    pub(crate) rules: Arc<[Arc<dyn Rule>]>,
    pub(crate) active_rule_ids: Vec<&'static str>,
}

impl ImportRuleModule {
    pub(crate) fn new() -> Self {
        let rules: Vec<Arc<dyn Rule>> = vec![Arc::new(ImportOrdering), Arc::new(WildcardImport)];
        let active_rule_ids = vec!["ImportOrdering", "WildcardImport"];
        Self {
            rules: rules.into(),
            active_rule_ids,
        }
    }
}
