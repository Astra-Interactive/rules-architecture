use crate::source::model::WorkspaceUseKind;
use crate::test::declaration::util::read_code_context;

#[test]
fn given_cfg_import_alias_argument_when_syntax_expanded_under_branch_then_only_that_branch_owns_the_argument()
 {
    let context = read_code_context(
        "mod left {pub struct Model;} mod right {pub struct Model;} #[cfg(unix)] use left::Model; #[cfg(not(unix))] use right::Model; type Shared<T>=std::sync::Arc<T>;",
    );
    let typed: syn::Type = syn::parse_str("Shared<Model>").unwrap();
    let cfg = crate::source::model::CfgPredicate::Not(Box::new(
        crate::source::model::CfgPredicate::Atom {
            key: "unix".to_owned(),
            value: None,
        },
    ));
    let expanded = context.resolve_type_syntax_under(0, 0, &typed, &cfg);
    assert_eq!(1, expanded.len());
    assert!(!expanded[0].has_unsubstituted_parameters);
    let targets =
        context.resolve_workspace_type(expanded[0].index, expanded[0].module, &expanded[0].typed);
    assert_eq!(1, targets.len());
    assert_eq!(
        vec!["right"],
        context.indexes[targets[0].index].modules[targets[0].path.module]
            .module
            .path
    );
    assert!(
        context
            .resolve_type_syntax(0, 0, &typed)
            .iter()
            .any(|syntax| syntax.has_unsubstituted_parameters)
    );
}

#[test]
fn given_generic_alias_constructor_and_external_wrapper_when_indexed_then_only_the_outer_constructed_type_is_a_dependency()
 {
    let context = read_code_context(
        "struct Dependency; impl Dependency {fn new()->Self{Self}} type Identity<T>=T; type Values=Vec<Dependency>; fn make(){let _=Identity::<Dependency>::new();let _=Values::new();}",
    );
    let item = context.indexes[0]
        .items
        .iter()
        .position(|item| item.name == "Dependency")
        .unwrap();
    let calls = context
        .name_index()
        .users_of(0, item)
        .filter(|usage| usage.kind == WorkspaceUseKind::Call)
        .collect::<Vec<_>>();
    assert_eq!(1, calls.len());
    assert_eq!(Some("Identity"), calls[0].path.first().map(String::as_str));
    let alias = context.indexes[0]
        .items
        .iter()
        .position(|item| item.name == "Identity")
        .unwrap();
    assert!(
        context
            .name_index()
            .users_of(0, alias)
            .any(|usage| usage.kind == WorkspaceUseKind::Call)
    );
}

#[test]
fn given_local_item_binders_when_indexed_then_generic_scope_does_not_hide_later_workspace_types() {
    let context = read_code_context(
        "struct T; fn call() { struct Local<T>(T); type Alias=Local<u8>; const SIZE:usize=size_of::<Alias>(); let _:Alias=Local(0); let _:T=T; let _=SIZE; }",
    );
    assert!(
        context.indexes[0].unanalysable.is_empty(),
        "{:?}",
        context.indexes[0].unanalysable
    );
    let target = context.indexes[0]
        .items
        .iter()
        .position(|item| item.name == "T")
        .unwrap();
    assert!(
        context
            .name_index()
            .users_of(0, target)
            .any(|usage| usage.kind == WorkspaceUseKind::Type)
    );
}

#[test]
fn given_cfg_disjoint_external_type_aliases_when_resolved_under_field_predicate_then_inactive_framework_is_excluded()
 {
    let context = read_code_context(
        "#[cfg(unix)] type Update=tokio::sync::watch::Sender<u64>; #[cfg(not(unix))] type Update=local::Sender; mod local {pub struct Sender;}",
    );
    let typed: syn::Type = syn::parse_str("Update").unwrap();
    let not_unix = crate::source::model::CfgPredicate::Not(Box::new(
        crate::source::model::CfgPredicate::Atom {
            key: "unix".to_owned(),
            value: None,
        },
    ));
    assert!(
        context
            .resolve_external_type_paths_under(0, 0, &typed, &not_unix)
            .is_empty()
    );
    assert!(
        context
            .resolve_outer_external_type_paths_under(0, 0, &typed, &not_unix)
            .is_empty()
    );
}

#[test]
fn given_generic_alias_type_lifetime_const_and_default_arguments_when_expanded_then_raw_shape_and_identities_are_preserved()
 {
    let context = read_code_context(
        "struct Model; type Shared<T>=std::sync::Arc<T>; type Borrowed<'a,T>=&'a T; type Bytes<const N:usize>=[u8;N]; type Defaulted<T=Model>=Option<T>;",
    );
    for written in ["Shared<Model>", "Defaulted"] {
        let typed: syn::Type = syn::parse_str(written).unwrap();
        let expanded = context.resolve_type_syntax(0, 0, &typed);
        assert_eq!(1, expanded.len());
        assert!(!expanded[0].has_unsubstituted_parameters);
        assert!(
            context
                .resolve_workspace_type(expanded[0].index, expanded[0].module, &expanded[0].typed)
                .iter()
                .any(|target| target
                    .path
                    .item
                    .is_some_and(|item| context.indexes[target.index].items[item].name == "Model"))
        );
    }
    let typed: syn::Type = syn::parse_str("Borrowed<'static,Model>").unwrap();
    let expanded = context.resolve_type_syntax(0, 0, &typed);
    assert!(
        matches!(&expanded[0].typed,syn::Type::Reference(reference) if reference.lifetime.as_ref().is_some_and(|lifetime|lifetime.ident=="static"))
    );
    assert!(!expanded[0].has_unsubstituted_parameters);
    let typed: syn::Type = syn::parse_str("Bytes<4>").unwrap();
    let expanded = context.resolve_type_syntax(0, 0, &typed);
    assert!(
        matches!(&expanded[0].typed,syn::Type::Array(array) if matches!(&array.len,syn::Expr::Lit(literal) if matches!(&literal.lit,syn::Lit::Int(number) if number.base10_digits()=="4")))
    );
    assert!(!expanded[0].has_unsubstituted_parameters);
}

#[test]
fn given_aliases_and_constructor_calls_when_indexed_then_exact_identity_and_member_are_retained() {
    let context = read_code_context(
        "pub struct Dependency; impl Dependency { fn new() -> Self { Self } fn helper() {} } type Alias = Dependency; fn new() { let _ = Alias::new(); Alias::helper(); } ",
    );
    let item = context.indexes[0]
        .items
        .iter()
        .position(|item| item.name == "Dependency")
        .unwrap();
    let uses = context.name_index().users_of(0, item).collect::<Vec<_>>();
    assert!(uses.iter().any(|usage| usage.kind == WorkspaceUseKind::Call
        && usage.path.last().is_some_and(|name| name == "new")
        && usage.function.as_deref() == Some("new")
        && usage.block.is_some()));
    assert!(uses.iter().any(|usage| usage.kind == WorkspaceUseKind::Call
        && usage.path.last().is_some_and(|name| name == "helper")));
    assert!(uses.iter().all(|usage| !usage.ambiguous && !usage.is_test));
}

#[test]
fn given_conditional_duplicate_names_when_indexed_then_disjoint_branches_are_not_ambiguous() {
    let context = read_code_context(
        "#[cfg(feature=\"native\")] struct Value; #[cfg(not(feature=\"native\"))] struct Value; fn consume(_: Value) {} #[cfg(test)] fn check(_: Value) {} ",
    );
    let uses = &context.name_index().uses;
    assert!(uses.iter().any(|usage| usage.is_test));
    assert!(uses.iter().any(|usage| !usage.is_test));
    assert!(uses.iter().all(|usage| !usage.ambiguous));
}

#[test]
fn given_aliases_and_nested_types_when_resolved_then_outer_query_and_reference_syntax_remain_distinct()
 {
    let context = read_code_context(
        "struct Value; type Wrapped<T> = Option<T>; type Borrowed<'a> = &'a Value; type Time = std::time::SystemTime;",
    );
    let wrapped: syn::Type = syn::parse_str("Wrapped<Value>").unwrap();
    assert!(
        context
            .resolve_workspace_type(0, 0, &wrapped)
            .iter()
            .any(|target| target
                .path
                .item
                .is_some_and(|item| context.indexes[target.index].items[item].name == "Value"))
    );
    let result: syn::Type = syn::parse_str("Result<Time, ()>").unwrap();
    assert!(
        context
            .resolve_external_type_paths(0, 0, &result)
            .iter()
            .any(|path| path.format_path() == "std::time::SystemTime")
    );
    assert!(
        !context
            .resolve_outer_external_type_paths(0, 0, &result)
            .iter()
            .any(|path| path.format_path() == "std::time::SystemTime")
    );
    let borrowed: syn::Type = syn::parse_str("Borrowed<'static>").unwrap();
    assert!(
        context
            .resolve_type_syntax(0, 0, &borrowed)
            .iter()
            .all(|syntax| matches!(syntax.typed, syn::Type::Reference(_)))
    );
}

pub(crate) fn read_resolution_context() -> crate::check::model::RuleContext {
    use crate::source::api::SourceReader;
    use crate::workspace::api::WorkspaceReader;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../cases/source/resolution-targets");
    let workspace = crate::workspace::cargo::CargoWorkspaceReader::new(
        crate::workspace::util::resolve_cargo_path(),
    )
    .read_workspace(&root)
    .unwrap();
    let config = crate::config::util::parse_project_config(&workspace).unwrap();
    let crates = crate::source::syn::SynSourceReader.read_sources(&workspace);
    assert!(crates.iter().all(|sources| sources.unanalysable.is_empty()));
    let indexes = crate::source::syn::CrateIndexBuilder::build_all(&crates, &workspace);
    for index in &indexes {
        assert!(index.unanalysable.is_empty(), "{:?}", index.unanalysable);
    }
    let owner = workspace
        .members
        .iter()
        .find(|member| member.name == "identity-owner")
        .unwrap();
    assert_eq!("digest_wire", owner.dependencies[0].code_name);
    crate::check::model::RuleContext {
        workspace,
        project_config: config,
        crates,
        indexes,
        names: std::sync::OnceLock::new(),
        rustfmt: crate::workspace::model::RustfmtFacts::NotQueried,
    }
}

#[test]
fn given_workspace_generic_alias_and_duplicate_model_names_when_expanded_then_argument_and_alias_body_keep_distinct_owners()
 {
    let context = read_resolution_context();
    let owner = context
        .indexes
        .iter()
        .position(|index| {
            index.crate_name == "identity-owner"
                && index.items.iter().any(|item| item.name == "Model")
        })
        .unwrap();
    let typed: syn::Type = syn::parse_str("digest_wire::Pair<Model>").unwrap();
    let expanded = context.resolve_type_syntax(owner, 0, &typed);
    assert_eq!(1, expanded.len());
    assert!(!expanded[0].has_unsubstituted_parameters);
    let targets =
        context.resolve_workspace_type(expanded[0].index, expanded[0].module, &expanded[0].typed);
    let owners = targets
        .iter()
        .map(|target| context.indexes[target.index].crate_name.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        std::collections::BTreeSet::from(["digest-package", "identity-owner"]),
        owners
    );
}

#[test]
fn given_real_library_target_names_and_explicit_test_root_when_read_then_sources_and_cross_target_uses_resolve()
 {
    let context = read_resolution_context();
    assert!(context.crates.iter().any(|sources| {
        sources
            .files
            .iter()
            .any(|file| file.path.ends_with("test/storage.rs"))
    }));
    assert!(context.name_index().uses.iter().any(
        |usage| context.indexes[usage.source_index].target
            != context.indexes[usage.target_index].target
            && usage.path.last().is_some_and(|name| name == "digest")
    ));
}

#[test]
fn given_independent_crate_features_and_test_modes_when_indexed_then_external_users_are_retained() {
    let context = read_resolution_context();
    let provider = context
        .indexes
        .iter()
        .position(|index| index.crate_name == "digest-package")
        .unwrap();
    for name in ["FeatureRecord", "FeatureAlias", "ProductionRecord"] {
        let identity = context.indexes[provider]
            .items
            .iter()
            .position(|item| item.name == name)
            .unwrap();
        let uses = context
            .name_index()
            .users_of(provider, identity)
            .collect::<Vec<_>>();
        assert!(
            uses.iter()
                .any(|usage| context.indexes[usage.source_index].crate_name == "identity-owner"),
            "{name}"
        );
        assert!(context.has_external_name_user(provider, identity), "{name}");
        assert!(
            uses.iter()
                .any(|usage| usage.is_test == (name == "ProductionRecord")),
            "{name}"
        );
    }
}
