use crate::check::api::Rule;
use crate::config::rule::LintConfig;
use crate::config::util::render_required_configuration;
use std::sync::Arc;

pub(crate) struct ConfigRuleModule {
    pub(crate) rules: Arc<[Arc<dyn Rule>]>,
    pub(crate) active_rule_ids: Vec<&'static str>,
}

impl ConfigRuleModule {
    pub(crate) fn new() -> Self {
        let rules: Vec<Arc<dyn Rule>> =
            vec![Arc::new(LintConfig::new(render_required_configuration()))];
        let active_rule_ids = vec!["LintConfig"];
        Self {
            rules: rules.into(),
            active_rule_ids,
        }
    }
}
