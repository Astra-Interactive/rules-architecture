use crate::check::api::Rule;
use crate::naming::rule::{
    ClosureParameterName, ExplicitItName, FunctionVerb, ImplementationName, KindNameInPackage,
    RolelessTypeName, TestFunctionName,
};
use std::sync::Arc;

pub(crate) struct NamingRuleModule {
    pub(crate) rules: Arc<[Arc<dyn Rule>]>,
    pub(crate) active_rule_ids: Vec<&'static str>,
}

impl NamingRuleModule {
    pub(crate) fn new() -> Self {
        let rules: Vec<Arc<dyn Rule>> = vec![
            Arc::new(ClosureParameterName),
            Arc::new(ExplicitItName),
            Arc::new(FunctionVerb),
            Arc::new(ImplementationName),
            Arc::new(KindNameInPackage),
            Arc::new(TestFunctionName),
            Arc::new(RolelessTypeName),
        ];
        let active_rule_ids = vec![
            "ClosureParameterName",
            "ExplicitItName",
            "FunctionVerb",
            "ImplementationName",
            "KindNameInPackage",
            "TestFunctionName",
            "RolelessTypeName",
        ];
        Self {
            rules: rules.into(),
            active_rule_ids,
        }
    }
}
