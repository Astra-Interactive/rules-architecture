use crate::check::api::Rule;
use crate::declaration::rule::{
    BareIdentifier, CallbackDeclaration, ContractShape, ErrorFamilyShape, LocalItem, ModelDerives,
    NamespaceType, ValueObjectShape,
};
use std::sync::Arc;

pub(crate) struct DeclarationRuleModule {
    pub(crate) rules: Arc<[Arc<dyn Rule>]>,
    pub(crate) active_rule_ids: Vec<&'static str>,
}

impl DeclarationRuleModule {
    pub(crate) fn new() -> Self {
        let rules: Vec<Arc<dyn Rule>> = vec![
            Arc::new(BareIdentifier),
            Arc::new(CallbackDeclaration),
            Arc::new(NamespaceType),
            Arc::new(LocalItem::new()),
            Arc::new(ModelDerives),
            Arc::new(ContractShape),
            Arc::new(ValueObjectShape),
            Arc::new(ErrorFamilyShape),
        ];
        let active_rule_ids = vec![
            "BareIdentifier",
            "CallbackDeclaration",
            "NamespaceType",
            "LocalItem",
            "ModelDerives",
            "ContractShape",
            "ValueObjectShape",
            "ErrorFamilyShape",
        ];
        Self {
            rules: rules.into(),
            active_rule_ids,
        }
    }
}
