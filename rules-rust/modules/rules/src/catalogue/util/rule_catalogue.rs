use crate::catalogue::model::AnchorKinds;
use crate::catalogue::model::CatalogueEntry;
use crate::catalogue::model::RetiredRule;

const NONE: u8 = 0;
const NODE: u8 = 1;
const MODULE: u8 = 2;
const FILE: u8 = 4;
const CRATE: u8 = 8;
const EDGE: u8 = 16;
const WORKSPACE: u8 = 32;

fn build_anchor_kinds(mask: u8) -> AnchorKinds {
    AnchorKinds {
        node: mask & NODE != 0,
        module: mask & MODULE != 0,
        file: mask & FILE != 0,
        krate: mask & CRATE != 0,
        workspace: mask & WORKSPACE != 0,
    }
}

fn build_entry(
    id: &'static str,
    phase: u8,
    mask: u8,
    reads_platform_crates: bool,
) -> CatalogueEntry {
    CatalogueEntry {
        id,
        phase,
        anchors: build_anchor_kinds(mask),
        reads_platform_crates,
    }
}

fn build_retired_rule(id: &'static str, replacement: &'static str) -> RetiredRule {
    RetiredRule { id, replacement }
}

fn build_workspace_entries() -> Vec<CatalogueEntry> {
    vec![
        build_entry("LintConfig", 1, NONE, false),
        build_entry("RootSources", 1, WORKSPACE, false),
        build_entry("UnitReadme", 1, CRATE, false),
        build_entry("NestedUnit", 1, CRATE, false),
        build_entry("ContainerSources", 1, WORKSPACE, false),
        build_entry("UnitPath", 1, CRATE, false),
        build_entry("ProjectDirMatchesPath", 1, WORKSPACE, false),
        build_entry("RootBuildSubprojects", 1, CRATE, false),
        build_entry("CrateTargets", 1, CRATE, false),
        build_entry("UnitDependencyCycle", 1, NONE, false),
        build_entry("EntryPointDependency", 1, NONE, false),
        build_entry("RootPackagePath", 2, CRATE | WORKSPACE, false),
        build_entry("PublishedVersion", 2, CRATE, false),
        build_entry("XtaskAlias", 2, WORKSPACE, false),
        build_entry("ShellAutomation", 2, WORKSPACE, false),
        build_entry("DependencyFloor", 2, WORKSPACE, false),
        build_entry("TimeCrate", 2, WORKSPACE, false),
        build_entry("PlatformSourceSetUnit", 2, CRATE, false),
        build_entry("UnitDependency", 2, EDGE, false),
        build_entry("ApiUnitDependency", 3, EDGE, false),
        build_entry("TechnologyDependency", 3, EDGE, false),
        build_entry("PluginImplUnit", 3, CRATE, false),
        build_entry("ApiUnitPlatformDependency", 3, EDGE, true),
        build_entry("PlatformDependencyTable", 3, EDGE, false),
        build_entry("DocHiddenUse", 4, NODE, false),
        build_entry("DiModuleVisibility", 4, NODE, false),
    ]
}

fn build_module_entries() -> Vec<CatalogueEntry> {
    vec![
        build_entry("RootFileShape", 1, NODE | FILE, false),
        build_entry("ModuleVisibility", 1, MODULE, false),
        build_entry("TestMirror", 1, NODE | MODULE | CRATE, false),
        build_entry("ForbiddenPackageName", 1, MODULE, false),
        build_entry("PlatformPackage", 1, MODULE, false),
        build_entry("ReExportShape", 2, NODE, false),
        build_entry("ImportThroughFileModule", 2, NODE, false),
        build_entry("ApiConfiguration", 2, NODE, false),
        build_entry("KindShape", 2, MODULE, false),
        build_entry("KindImportDirection", 2, NODE, false),
        build_entry("PackageDependencyCycle", 2, NODE, false),
        build_entry("PlatformImport", 3, NODE, true),
        build_entry("ContractPlacement", 2, NODE, false),
        build_entry("UseCaseShape", 2, NODE, false),
        build_entry("UseCaseAbstraction", 3, NODE, false),
        build_entry("UtilShape", 2, NODE, false),
        build_entry("UtilFileName", 1, NODE | MODULE, false),
        build_entry("UtilFunctionAsExtension", 2, NODE, false),
        build_entry("PluralPackageName", 2, MODULE, false),
        build_entry("DiPlacement", 2, NODE | MODULE, false),
        build_entry("RootModulePlacement", 2, NODE, false),
        build_entry("EntryPointShape", 2, NODE | MODULE, false),
        build_entry("EntryPointVisibility", 2, NODE, false),
        build_entry("ImplementationVisibility", 2, NODE, false),
        build_entry("CoreShape", 3, NODE | MODULE, false),
        build_entry("ContractUnitShape", 3, NODE, false),
        build_entry("ErrorPlacement", 2, NODE, false),
        build_entry("EnumFamilyFile", 2, NODE, false),
        build_entry("NestedDataStruct", 4, NODE, false),
        build_entry("NestedEnum", 4, NODE, false),
        build_entry("FakePlacement", 1, NODE | CRATE, false),
        build_entry("InMemoryPlacement", 2, NODE, false),
        build_entry("ImplPlacement", 2, NODE, false),
        build_entry("UnsafePlacement", 2, NODE, false),
        build_entry("ListenerName", 2, NODE, false),
        build_entry("EventPlacement", 2, NODE, false),
        build_entry("IntraDocLink", 4, NODE, false),
        build_entry("VisibilityTier", 2, NODE, false),
        build_entry("AssociatedFactoryFunction", 2, NODE, false),
        build_entry("ModelContainsFunctions", 2, NODE, false),
        build_entry("ModelShouldBeImmutable", 2, NODE, false),
        build_entry("ForbiddenImport", 3, NODE, false),
        build_entry("KindContent", 2, NODE, false),
        build_entry("ImplementationPlacement", 2, NODE, false),
        build_entry("ForeignErrorType", 2, NODE, false),
    ]
}

fn build_code_style_entries() -> Vec<CatalogueEntry> {
    vec![
        build_entry("ForbiddenComment", 1, NONE, false),
        build_entry("ClosureParameterName", 1, NODE, false),
        build_entry("ExplicitItName", 1, NODE, false),
        build_entry("FunctionVerb", 1, NODE, false),
        build_entry("ImplementationName", 1, NODE, false),
        build_entry("RolelessTypeName", 2, NODE, false),
        build_entry("KindNameInPackage", 1, NODE, false),
        build_entry("BareIdentifier", 2, NODE, false),
        build_entry("TestFunctionName", 1, NODE, false),
        build_entry("CallbackDeclaration", 2, NODE, false),
        build_entry("DeclarationOrder", 1, NODE, false),
        build_entry("CalleeBeforeCaller", 1, NODE, false),
        build_entry("OneTypeFamilyPerFile", 1, NODE | MODULE, false),
        build_entry("ConstantOutsideImpl", 1, NODE, false),
        build_entry("NamespaceType", 2, NODE, false),
        build_entry("LocalItem", 2, NODE, false),
        build_entry("StructDefaultValue", 1, NODE, false),
        build_entry("AnonymousTuple", 1, NODE, false),
        build_entry("WildcardImport", 1, NODE, false),
        build_entry("PlatformFileSuffix", 2, NODE, false),
        build_entry("RustdocHtmlTag", 2, NODE, false),
        build_entry("RustdocFiller", 2, NODE, false),
        build_entry("BooleanOperatorWrapping", 2, NODE, false),
        build_entry("TypeAliasOnOneLine", 2, NODE, false),
        build_entry("InitializerOnDeclarationLine", 2, NODE, false),
        build_entry("ClosureSingleIfGuard", 2, NODE, false),
        build_entry("OperatorBeforeTrailingCall", 2, NODE, false),
        build_entry("TrivialClosurePath", 4, NODE, false),
        build_entry("ImportOrdering", 1, NODE, false),
        build_entry("BlockOnOutsideBoundary", 2, NODE, false),
        build_entry("BlockOnInTest", 2, NODE, false),
        build_entry("IgnoredTest", 2, NODE, false),
        build_entry("ExpectScope", 1, NODE, false),
    ]
}

fn build_code_behaviour_entries() -> Vec<CatalogueEntry> {
    vec![
        build_entry("DetachedTask", 4, NODE, false),
        build_entry("WatchTimeout", 4, NODE, false),
        build_entry("ImpureUpdateClosure", 4, NODE, false),
        build_entry("CallbackBridgeShape", 4, NODE, false),
        build_entry("BridgeCleanup", 4, NODE, false),
        build_entry("ThreadPrimitive", 4, NODE, false),
        build_entry("DurationOverloadCall", 4, NODE, false),
        build_entry("ResultConsumption", 4, NODE, false),
        build_entry("DiModuleLifecycle", 4, NODE, false),
        build_entry("RootModuleConstruction", 4, NODE, false),
        build_entry("SystemClockOutsideDi", 4, NODE, false),
        build_entry("TimePrimitive", 4, NODE, false),
        build_entry("ModelDerives", 2, NODE, false),
        build_entry("ValueObjectShape", 4, NODE, false),
        build_entry("ValidatedModelDerives", 4, NODE, false),
        build_entry("ContractShape", 2, NODE, false),
        build_entry("ErrorFamilyShape", 4, NODE, false),
        build_entry("ErrorLogging", 4, NODE, false),
        build_entry("CatchUnwind", 4, NODE, false),
        build_entry("DocCommentScope", 4, NODE, false),
        build_entry("GlobalState", 4, NODE, false),
        build_entry("DiModuleShape", 4, NODE, false),
        build_entry("ConstructorDependency", 4, NODE, false),
        build_entry("MainShape", 4, NODE, false),
        build_entry("BlockingCallInAsync", 4, NODE, false),
        build_entry("BroadcastLag", 4, NODE, false),
        build_entry("SelectCancelSafety", 4, NODE, false),
        build_entry("JoinSetBehindLock", 4, NODE, false),
        build_entry("BlockingLockInSync", 4, NODE, false),
        build_entry("CriticalSectionWrite", 4, NODE, false),
        build_entry("LockChoice", 4, NODE, false),
        build_entry("TestTimePaused", 4, NODE, false),
        build_entry("ModelRuleTest", 4, NODE, false),
    ]
}

fn build_meta_entries() -> Vec<CatalogueEntry> {
    vec![
        build_entry("Unanalysable", 1, NODE | CRATE, false),
        build_entry("UnfulfilledExpectation", 1, NONE, false),
        build_entry("UnknownRule", 1, NONE, false),
        build_entry("RetiredRule", 1, NONE, false),
        build_entry("MalformedExpectation", 1, NONE, false),
    ]
}

fn build_retired_rules() -> Vec<RetiredRule> {
    vec![
        build_retired_rule("ActualFileSuffix", "PlatformFileSuffix"),
        build_retired_rule("CallbackFlowShape", "CallbackBridgeShape"),
        build_retired_rule("ClockSystemOutsideDi", "SystemClockOutsideDi"),
        build_retired_rule("CompanionFactoryFunction", "AssociatedFactoryFunction"),
        build_retired_rule("ConstantOutsideCompanion", "ConstantOutsideImpl"),
        build_retired_rule("DataClassContainsFunctions", "ModelContainsFunctions"),
        build_retired_rule("DataClassDefaultValue", "StructDefaultValue"),
        build_retired_rule("DataClassShouldBeImmutable", "ModelShouldBeImmutable"),
        build_retired_rule("GlobalCoroutineUsage", "DetachedTask"),
        build_retired_rule("ImplicitItParameter", "ClosureParameterName"),
        build_retired_rule("ImpureUpdateLambda", "ImpureUpdateClosure"),
        build_retired_rule("KDocFiller", "RustdocFiller"),
        build_retired_rule("KDocHtmlTag", "RustdocHtmlTag"),
        build_retired_rule("LambdaSingleIfGuard", "ClosureSingleIfGuard"),
        build_retired_rule("LocalOrInnerClass", "LocalItem"),
        build_retired_rule("NamespaceObject", "NamespaceType"),
        build_retired_rule("NestedDataClass", "NestedDataStruct"),
        build_retired_rule("RedundantVisibilityModifier", "VisibilityTier"),
        build_retired_rule("RolelessClassName", "RolelessTypeName"),
        build_retired_rule("RunBlockingInTest", "BlockOnInTest"),
        build_retired_rule("RunBlockingOutsideBoundary", "BlockOnOutsideBoundary"),
        build_retired_rule("SealedFamilyFile", "EnumFamilyFile"),
        build_retired_rule("StateFlowTimeout", "WatchTimeout"),
        build_retired_rule("TrivialLambdaReference", "TrivialClosurePath"),
    ]
}

pub(crate) fn find_catalogue_entry(id: &str) -> Option<CatalogueEntry> {
    [
        build_workspace_entries(),
        build_module_entries(),
        build_code_style_entries(),
        build_code_behaviour_entries(),
        build_meta_entries(),
    ]
    .into_iter()
    .flatten()
    .find(|entry| entry.id == id)
}

pub(crate) fn find_retired_rule(id: &str) -> Option<RetiredRule> {
    build_retired_rules().into_iter().find(|rule| rule.id == id)
}
