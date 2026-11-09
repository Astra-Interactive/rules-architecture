use crate::check::api::Rule;
use crate::expression::rule::{
    BooleanOperatorWrapping, ClosureSingleIfGuard, OperatorBeforeTrailingCall, TrivialClosurePath,
};
use std::sync::Arc;

pub(crate) struct ExpressionRuleModule {
    pub(crate) rules: Arc<[Arc<dyn Rule>]>,
    pub(crate) active_rule_ids: Vec<&'static str>,
}

impl ExpressionRuleModule {
    pub(crate) fn new() -> Self {
        let rules: Vec<Arc<dyn Rule>> = vec![
            Arc::new(BooleanOperatorWrapping),
            Arc::new(ClosureSingleIfGuard),
            Arc::new(OperatorBeforeTrailingCall),
            Arc::new(TrivialClosurePath),
        ];
        let active_rule_ids = vec![
            "BooleanOperatorWrapping",
            "ClosureSingleIfGuard",
            "OperatorBeforeTrailingCall",
            "TrivialClosurePath",
        ];
        Self {
            rules: rules.into(),
            active_rule_ids,
        }
    }
}
