use crate::check::api::Rule;
use crate::module::rule::{
    ContractPlacement, ContractUnitShape, CoreShape, DiPlacement, EntryPointShape,
    EntryPointVisibility, ForbiddenPackageName, ForeignErrorType, ImplementationPlacement,
    ImplementationVisibility, KindContent, KindShape, PluralPackageName, RootModulePlacement,
    UseCaseAbstraction, UseCaseShape, UtilFunctionAsExtension, UtilShape,
};
use std::sync::Arc;

pub(crate) struct ModuleShapeRuleModule {
    pub(crate) rules: Arc<[Arc<dyn Rule>]>,
    pub(crate) active_rule_ids: Vec<&'static str>,
}

impl ModuleShapeRuleModule {
    pub(crate) fn new() -> Self {
        let rules: Vec<Arc<dyn Rule>> = vec![
            Arc::new(KindShape),
            Arc::new(ContractPlacement),
            Arc::new(UseCaseShape),
            Arc::new(UtilShape),
            Arc::new(UtilFunctionAsExtension),
            Arc::new(PluralPackageName),
            Arc::new(DiPlacement),
            Arc::new(RootModulePlacement),
            Arc::new(EntryPointShape),
            Arc::new(EntryPointVisibility),
            Arc::new(ImplementationVisibility),
            Arc::new(UseCaseAbstraction),
            Arc::new(CoreShape),
            Arc::new(ContractUnitShape),
            Arc::new(ForbiddenPackageName),
            Arc::new(KindContent),
            Arc::new(ImplementationPlacement),
            Arc::new(ForeignErrorType),
        ];
        let active_rule_ids = vec![
            "KindShape",
            "ContractPlacement",
            "UseCaseShape",
            "UtilShape",
            "UtilFunctionAsExtension",
            "PluralPackageName",
            "DiPlacement",
            "RootModulePlacement",
            "EntryPointShape",
            "EntryPointVisibility",
            "ImplementationVisibility",
            "UseCaseAbstraction",
            "CoreShape",
            "ContractUnitShape",
            "ForbiddenPackageName",
            "KindContent",
            "ImplementationPlacement",
            "ForeignErrorType",
        ];
        Self {
            rules: rules.into(),
            active_rule_ids,
        }
    }
}
