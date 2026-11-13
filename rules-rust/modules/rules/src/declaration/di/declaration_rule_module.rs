use crate::check::api::Rule;
use crate::declaration::rule::{
    BareIdentifier, CallbackDeclaration, ContractShape, ErrorFamilyShape,
    InitializerOnDeclarationLine, LocalItem, ModelDerives, NamespaceType, TypeAliasOnOneLine,
    ValidatedModelDerives, ValueObjectShape,
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
            Arc::new(ValidatedModelDerives),
            Arc::new(ErrorFamilyShape),
            Arc::new(TypeAliasOnOneLine),
            Arc::new(InitializerOnDeclarationLine),
        ];
        let active_rule_ids = vec![
            "BareIdentifier",
            "CallbackDeclaration",
            "NamespaceType",
            "LocalItem",
            "ModelDerives",
            "ContractShape",
            "ValueObjectShape",
            "ValidatedModelDerives",
            "ErrorFamilyShape",
            "TypeAliasOnOneLine",
            "InitializerOnDeclarationLine",
        ];
        Self {
            rules: rules.into(),
            active_rule_ids,
        }
    }
}
