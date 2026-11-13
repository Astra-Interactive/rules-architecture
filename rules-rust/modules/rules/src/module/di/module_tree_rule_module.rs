use crate::check::api::Rule;
use crate::module::rule::{
    ApiConfiguration, ForbiddenImport, ImportThroughFileModule, KindImportDirection,
    ModuleVisibility, PackageDependencyCycle, PlatformFileSuffix, PlatformImport, PlatformPackage,
    ReExportShape, RootFileShape, TestMirror, UtilFileName,
};
use std::sync::Arc;

pub(crate) struct ModuleTreeRuleModule {
    pub(crate) rules: Arc<[Arc<dyn Rule>]>,
    pub(crate) active_rule_ids: Vec<&'static str>,
}

impl ModuleTreeRuleModule {
    pub(crate) fn new() -> Self {
        let rules: Vec<Arc<dyn Rule>> = vec![
            Arc::new(ModuleVisibility),
            Arc::new(PlatformPackage),
            Arc::new(RootFileShape),
            Arc::new(TestMirror),
            Arc::new(UtilFileName),
            Arc::new(PlatformFileSuffix::new()),
            Arc::new(ApiConfiguration),
            Arc::new(ReExportShape),
            Arc::new(ImportThroughFileModule),
            Arc::new(KindImportDirection),
            Arc::new(PackageDependencyCycle),
            Arc::new(PlatformImport),
            Arc::new(ForbiddenImport),
        ];
        let active_rule_ids = vec![
            "ModuleVisibility",
            "PlatformPackage",
            "RootFileShape",
            "TestMirror",
            "UtilFileName",
            "PlatformFileSuffix",
            "ApiConfiguration",
            "ReExportShape",
            "ImportThroughFileModule",
            "KindImportDirection",
            "PackageDependencyCycle",
            "PlatformImport",
            "ForbiddenImport",
        ];
        Self {
            rules: rules.into(),
            active_rule_ids,
        }
    }
}
