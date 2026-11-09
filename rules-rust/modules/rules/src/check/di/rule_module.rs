use crate::check::api::Rule;
use crate::comment::rule::ForbiddenComment;
use crate::config::rule::LintConfig;
use crate::config::util::render_required_configuration;
use crate::default::rule::StructDefaultValue;
use crate::dependency::rule::{EntryPointDependency, UnitDependencyCycle};
use crate::family::rule::{ConstantPlacement, OneTypeFamilyPerFile};
use crate::import::rule::{ImportOrdering, WildcardImport};
use crate::layout::rule::{
    ContainerSources, CrateTargets, NestedUnit, ProjectDirMatchesPath, RootBuildSubprojects,
    RootSources, UnitPath,
};
use crate::module::rule::{
    FakePlacement, ForbiddenPackageName, ModuleVisibility, PlatformPackage, RootFileShape,
    TestMirror, UtilFileName,
};
use crate::naming::rule::{
    ClosureParameterName, ExplicitItName, FunctionVerb, ImplementationName, KindNameInPackage,
    TestFunctionName,
};
use crate::ordering::rule::{CalleeBeforeCaller, DeclarationOrder};
use crate::readme::rule::UnitReadme;
use crate::suppression::rule::ExpectScope;
use crate::tuple::rule::AnonymousTuple;
use std::sync::Arc;

pub(crate) struct RuleModule {
    pub(crate) rules: Vec<Arc<dyn Rule>>,
    pub(crate) active_rule_ids: Vec<&'static str>,
}

impl RuleModule {
    pub(crate) fn new() -> Self {
        let mut module = Self {
            rules: Vec::new(),
            active_rule_ids: Vec::new(),
        };
        module.register("AnonymousTuple", AnonymousTuple);
        module.register("CalleeBeforeCaller", CalleeBeforeCaller);
        module.register("ClosureParameterName", ClosureParameterName);
        module.register("ConstantOutsideImpl", ConstantPlacement);
        module.register("ContainerSources", ContainerSources);
        module.register("CrateTargets", CrateTargets);
        module.register("DeclarationOrder", DeclarationOrder);
        module.register("EntryPointDependency", EntryPointDependency);
        module.register("ExpectScope", ExpectScope);
        module.register("ExplicitItName", ExplicitItName);
        module.register("FakePlacement", FakePlacement);
        module.register("ForbiddenComment", ForbiddenComment);
        module.register("ForbiddenPackageName", ForbiddenPackageName);
        module.register("FunctionVerb", FunctionVerb);
        module.register("ImplementationName", ImplementationName);
        module.register("ImportOrdering", ImportOrdering);
        module.register("KindNameInPackage", KindNameInPackage);
        module.register(
            "LintConfig",
            LintConfig::new(render_required_configuration()),
        );
        module.register("ModuleVisibility", ModuleVisibility);
        module.register("NestedUnit", NestedUnit);
        module.register("OneTypeFamilyPerFile", OneTypeFamilyPerFile);
        module.register("PlatformPackage", PlatformPackage);
        module.register("ProjectDirMatchesPath", ProjectDirMatchesPath);
        module.register("RootBuildSubprojects", RootBuildSubprojects);
        module.register("RootFileShape", RootFileShape);
        module.register("RootSources", RootSources);
        module.register("StructDefaultValue", StructDefaultValue);
        module.register("TestFunctionName", TestFunctionName);
        module.register("TestMirror", TestMirror);
        module.register("UnitDependencyCycle", UnitDependencyCycle);
        module.register("UnitPath", UnitPath);
        module.register("UnitReadme", UnitReadme);
        module.register("UtilFileName", UtilFileName);
        module.register("WildcardImport", WildcardImport);
        module
    }

    fn register(&mut self, id: &'static str, rule: impl Rule + 'static) {
        self.rules.push(Arc::new(rule));
        self.active_rule_ids.push(id);
    }
}
