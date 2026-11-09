use crate::check::api::Rule;
use crate::module::rule::{
    AssociatedFactoryFunction, DiModuleVisibility, DocHiddenUse, EnumFamilyFile, ErrorPlacement,
    EventPlacement, FakePlacement, ImplPlacement, InMemoryPlacement, ListenerName,
    ModelContainsFunctions, ModelShouldBeImmutable, NestedDataStruct, NestedEnum, UnsafePlacement,
    VisibilityTier,
};
use std::sync::Arc;

pub(crate) struct ModuleOwnershipRuleModule {
    pub(crate) rules: Arc<[Arc<dyn Rule>]>,
    pub(crate) active_rule_ids: Vec<&'static str>,
}

impl ModuleOwnershipRuleModule {
    pub(crate) fn new() -> Self {
        let rules: Vec<Arc<dyn Rule>> = vec![
            Arc::new(FakePlacement),
            Arc::new(DocHiddenUse),
            Arc::new(DiModuleVisibility),
            Arc::new(NestedDataStruct),
            Arc::new(NestedEnum),
            Arc::new(AssociatedFactoryFunction),
            Arc::new(InMemoryPlacement),
            Arc::new(ModelShouldBeImmutable),
            Arc::new(ModelContainsFunctions),
            Arc::new(ListenerName),
            Arc::new(EventPlacement),
            Arc::new(UnsafePlacement),
            Arc::new(ErrorPlacement),
            Arc::new(EnumFamilyFile),
            Arc::new(ImplPlacement),
            Arc::new(VisibilityTier),
        ];
        let active_rule_ids = vec![
            "FakePlacement",
            "DocHiddenUse",
            "DiModuleVisibility",
            "NestedDataStruct",
            "NestedEnum",
            "AssociatedFactoryFunction",
            "InMemoryPlacement",
            "ModelShouldBeImmutable",
            "ModelContainsFunctions",
            "ListenerName",
            "EventPlacement",
            "UnsafePlacement",
            "ErrorPlacement",
            "EnumFamilyFile",
            "ImplPlacement",
            "VisibilityTier",
        ];
        Self {
            rules: rules.into(),
            active_rule_ids,
        }
    }
}
