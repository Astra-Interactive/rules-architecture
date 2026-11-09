use crate::check::api::Rule;
use crate::family::rule::{ConstantPlacement, OneTypeFamilyPerFile};
use std::sync::Arc;

pub(crate) struct FamilyRuleModule {
    pub(crate) rules: Arc<[Arc<dyn Rule>]>,
    pub(crate) active_rule_ids: Vec<&'static str>,
}

impl FamilyRuleModule {
    pub(crate) fn new() -> Self {
        let rules: Vec<Arc<dyn Rule>> =
            vec![Arc::new(ConstantPlacement), Arc::new(OneTypeFamilyPerFile)];
        let active_rule_ids = vec!["ConstantOutsideImpl", "OneTypeFamilyPerFile"];
        Self {
            rules: rules.into(),
            active_rule_ids,
        }
    }
}
