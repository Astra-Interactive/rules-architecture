use crate::source::model::CrateIndex;
use crate::source::model::Resolution;
use crate::source::model::SourceTree;
use crate::source::syn::CrateIndexBuilder;
use crate::source::syn::ModuleTreeWalker;
use crate::test::declaration::util::read_code_context;
use crate::workspace::api::WorkspaceReader;
use crate::workspace::cargo::CargoWorkspaceReader;
use crate::workspace::util::resolve_cargo_path;
use std::collections::BTreeMap;
use std::fs;

fn build_quotation_dependency_index(package: &str, dependency: &str, source: &str) -> CrateIndex {
    let directory = tempfile::tempdir().expect("temporary workspace");
    fs::create_dir_all(directory.path().join("provider/src")).expect("provider directory");
    fs::write(directory.path().join("provider/src/lib.rs"), "").expect("provider source");
    fs::write(
        directory.path().join("provider/Cargo.toml"),
        format!("[package]\nname = '{package}'\nversion = '0.1.0'\n"),
    )
    .expect("provider manifest");
    fs::write(
        directory.path().join("Cargo.toml"),
        format!("[package]\nname = 'sample'\nversion = '0.1.0'\n[workspace]\n[dependencies]\n{dependency} = {{ package = '{package}', path = 'provider' }}\n"),
    ).expect("consumer manifest");
    fs::create_dir(directory.path().join("src")).expect("consumer directory");
    fs::write(directory.path().join("src/lib.rs"), source).expect("consumer source");
    let workspace = CargoWorkspaceReader::new(resolve_cargo_path())
        .read_workspace(directory.path())
        .expect("valid dependency workspace");
    let sources = ModuleTreeWalker::new(
        directory.path().to_owned(),
        "sample".to_owned(),
        "sample".to_owned(),
    )
    .walk_crate("src/lib.rs", SourceTree::Src);
    CrateIndexBuilder::build_all(&[sources], &workspace).remove(0)
}

#[test]
fn given_cargo_quotation_packages_when_indexed_then_only_span_and_interpolation_are_dependencies() {
    for package in ["syn", "quote"] {
        let plain = if package == "syn" {
            "parse_quote"
        } else {
            "quote"
        };
        let spanned = format!("{plain}_spanned");
        for dependency in [package, "tokens"] {
            let source = format!(
                "use {dependency}::{plain} as render;\nmod leaf {{ pub fn quoted() {{}} pub fn span() -> u8 {{ 0 }} pub const INTERPOLATED: u8 = 0; }}\nuse crate::leaf::INTERPOLATED;\nfn run() {{\n{dependency}::{plain}!(tokio::runtime::Runtime);\nrender!(crate::leaf::quoted());\n{dependency}::{spanned}!(crate::leaf::span() => crate::leaf::quoted());\nrender!(#INTERPOLATED #( #INTERPOLATED )*);\nrender!(#missing);\n}}"
            );
            let index = build_quotation_dependency_index(package, dependency, &source);
            let subjects: Vec<_> = index
                .unanalysable
                .iter()
                .map(|site| site.subject.as_str())
                .collect();
            assert_eq!(subjects, ["missing"], "{package} as {dependency}");
            assert!(
                !index
                    .edges
                    .iter()
                    .any(|edge| edge.path.last().is_some_and(|name| name == "quoted"))
            );
            assert!(index.edges.iter().any(|edge| edge.span.start.line == 7
                && edge.path.last().is_some_and(|name| name == "span")));
            assert_eq!(
                2,
                index
                    .edges
                    .iter()
                    .filter(|edge| edge.span.start.line == 8
                        && edge.path.last().is_some_and(|name| name == "INTERPOLATED"))
                    .count()
            );
        }
    }
}

#[test]
fn given_custom_packages_with_quotation_macro_names_when_indexed_then_keeps_argument_dependencies()
{
    for (dependency, name) in [
        ("syn", "parse_quote"),
        ("syn", "parse_quote_spanned"),
        ("quote", "quote"),
        ("quote", "quote_spanned"),
    ] {
        let source = format!(
            "use {dependency}::{name} as render; mod leaf {{ pub fn quoted() {{}} }} fn run() {{ {dependency}::{name}!(tokio::runtime::Runtime); render!(crate::leaf::quoted()); }}"
        );
        let index = build_quotation_dependency_index("custom", dependency, &source);
        assert_eq!(1, index.unanalysable.len(), "{dependency}::{name}");
        assert!(index.unanalysable[0].subject.contains("tokio"));
        assert!(
            index
                .edges
                .iter()
                .any(|edge| edge.path.last().is_some_and(|name| name == "quoted"))
        );
    }
}

#[test]
fn given_cargo_renamed_loggers_when_indexed_then_package_identity_controls_field_grammar() {
    for package in ["tracing", "log", "custom"] {
        let directory = tempfile::tempdir().expect("temporary workspace");
        fs::create_dir_all(directory.path().join("provider/src")).expect("provider directory");
        fs::write(directory.path().join("provider/src/lib.rs"), "").expect("provider source");
        fs::write(
            directory.path().join("provider/Cargo.toml"),
            format!("[package]\nname = '{package}'\nversion = '0.1.0'\n"),
        )
        .expect("provider manifest");
        fs::write(
            directory.path().join("Cargo.toml"),
            format!("[package]\nname = 'sample'\nversion = '0.1.0'\n[workspace]\n[dependencies]\ntelemetry = {{ package = '{package}', path = 'provider' }}\n"),
        )
        .expect("consumer manifest");
        fs::create_dir(directory.path().join("src")).expect("consumer directory");
        fs::write(
            directory.path().join("src/lib.rs"),
            "use telemetry::info as record; fn run(value: u64) { telemetry::info!(direct = value); record!(aliased = value); record!(broken = missing); }",
        )
        .expect("consumer source");
        let workspace = CargoWorkspaceReader::new(resolve_cargo_path())
            .read_workspace(directory.path())
            .expect("valid renamed dependency workspace");
        let sources = ModuleTreeWalker::new(
            directory.path().to_owned(),
            "sample".to_owned(),
            "sample".to_owned(),
        )
        .walk_crate("src/lib.rs", SourceTree::Src);
        let indexes = CrateIndexBuilder::build_all(&[sources], &workspace);
        let subjects = indexes[0]
            .unanalysable
            .iter()
            .map(|site| site.subject.as_str())
            .collect::<Vec<_>>();
        if package == "custom" {
            assert_eq!(subjects, ["direct", "aliased", "broken", "missing"]);
        } else {
            assert_eq!(subjects, ["missing"]);
        }
    }
}

#[test]
fn given_logger_field_labels_when_indexed_then_only_field_values_are_variable_references() {
    let context = read_code_context(
        "use tracing::info as record; fn run(value: u64) { record!(field = value); tracing::info!(other = missing); custom!(unknown = value); }",
    );
    let subjects = context.indexes[0]
        .unanalysable
        .iter()
        .map(|site| site.subject.as_str())
        .collect::<Vec<_>>();
    assert_eq!(subjects, ["missing", "unknown"]);
}

#[test]
fn given_absolute_external_paths_and_local_shadowing_when_resolved_then_external_roots_keep_their_namespace()
 {
    let index = build_index(
        "mod std { pub struct Value; } use ::std::time::Duration as External; fn call() { External::ZERO; std::Value; ::std::time::Duration::ZERO; }",
    );
    assert!(matches!(
        resolve(&index, 0, "std::Value"),
        Resolution::Internal(_)
    ));
    assert_eq!(
        resolve(&index, 0, "::std::time::Duration"),
        Resolution::External("std::time::Duration".to_owned())
    );
    assert!(index.unanalysable.is_empty());
}

pub(crate) fn build_index(source: &str) -> CrateIndex {
    build_index_files(source, BTreeMap::new())
}

pub(crate) fn build_index_files(source: &str, files: BTreeMap<&str, &str>) -> CrateIndex {
    let directory = tempfile::tempdir().expect("temporary source root");
    fs::write(directory.path().join("lib.rs"), source).expect("root source");
    for (path, text) in files {
        let path = directory.path().join(path);
        fs::create_dir_all(path.parent().expect("parent path")).expect("source parent");
        fs::write(path, text).expect("module source");
    }
    let sources = ModuleTreeWalker::new(
        directory.path().to_owned(),
        "sample".to_owned(),
        "sample".to_owned(),
    )
    .walk_crate("lib.rs", SourceTree::Src);
    CrateIndexBuilder::new(&sources, vec!["renamed_dependency".to_owned()]).build()
}

fn resolve(index: &CrateIndex, module: usize, path: &str) -> Resolution {
    index.resolve_path(module, &syn::parse_str(path).expect("valid path"))
}

#[test]
fn given_aliases_reexports_and_globs_when_resolved_then_defining_modules_and_external_names_are_preserved()
 {
    let index = build_index(
        "mod source { pub struct Thing; } mod bridge { pub use crate::source::Thing as Alias; } mod consumer { use crate::bridge::Alias as Local; use crate::source::*; fn use_items() { Local; Thing; renamed_dependency::Type::new(); } }",
    );
    let consumer = index
        .modules
        .iter()
        .find(|module| module.module.path == ["consumer"])
        .expect("consumer")
        .id;
    let Resolution::Internal(aliases) = resolve(&index, consumer, "Local") else {
        panic!("alias resolves");
    };
    let Resolution::Internal(globs) = resolve(&index, consumer, "Thing") else {
        panic!("glob resolves");
    };
    assert_eq!(aliases, globs);
    assert_eq!(index.modules[aliases[0].module].module.path, ["source"]);
    assert_eq!(
        resolve(&index, consumer, "renamed_dependency::Type"),
        Resolution::External("renamed_dependency::Type".to_owned())
    );
    assert!(index.unanalysable.is_empty());
}

#[test]
fn given_shadowed_names_generics_patterns_and_macro_arguments_when_indexed_then_only_unknown_paths_are_reported()
 {
    let index = build_index(
        "mod item { pub fn run() {} } fn inspect<T>(input: Option<T>) { let item = input; item; if let Some(value) = item { value; } match item { Some(value) if true => { value; }, _ => () } let closure = |argument: T| argument; closure; for element in Vec::<T>::new() { element; } assert!(missing::run()); }",
    );
    let subjects: Vec<&str> = index
        .unanalysable
        .iter()
        .map(|site| site.subject.as_str())
        .collect();
    assert_eq!(subjects, ["missing :: run"]);
    assert!(index.edges.iter().all(|edge| edge.target != 1));
    assert!(matches!(
        index.unanalysable[0].site,
        crate::source::model::UnanalysableSite::Node { .. }
    ));
}

#[test]
fn given_cfg_sibling_modules_and_test_sources_when_indexed_then_occurrences_and_predicates_are_distinct()
 {
    let index = build_index(
        "#[cfg(target_os = \"linux\")] mod variant { pub struct Thing; } #[cfg(target_os = \"windows\")] mod variant { pub struct Thing; } mod production { fn call() { crate::variant::Thing; } } #[cfg(test)] mod test { fn check() { crate::variant::Thing; } }",
    );
    let variants: Vec<_> = index
        .modules
        .iter()
        .filter(|module| module.module.path == ["variant"])
        .collect();
    assert_eq!(variants.len(), 2);
    assert_ne!(variants[0].id, variants[1].id);
    assert!(!variants[0].cfg.and(&variants[1].cfg).is_satisfiable());
    assert!(index.edges.iter().any(|edge| edge.is_test));
    assert!(index.edges.iter().any(|edge| !edge.is_test));
}

#[test]
fn given_external_glob_and_broken_alias_cycle_when_resolved_then_only_the_cycle_fails_closed() {
    let index = build_index(
        "mod consumer { use renamed_dependency::*; fn call() { Unknown::new(); } } mod broken { use self::Other as Alias; use self::Alias as Other; fn call() { Alias::new(); } }",
    );
    assert_eq!(resolve(&index, 1, "Unknown::new"), Resolution::ExternalGlob);
    assert_eq!(resolve(&index, 2, "Alias::new"), Resolution::Unresolved);
    assert!(
        index
            .unanalysable
            .iter()
            .all(|site| !site.subject.contains("Unknown"))
    );
    assert!(!index.unanalysable.is_empty());
}

#[test]
fn given_trait_alias_function_and_impl_when_indexed_then_complete_syntax_is_available_to_rule_consumers()
 {
    let index = build_index(
        "trait Base {} #[custom] trait Contract<T>: Base { fn execute(&mut self, input: T) -> T; } type Alias = u32; fn execute(input: Alias) -> Alias { input } struct Value; impl Contract<u32> for Value { fn execute(&mut self, input: u32) -> u32 { input } }",
    );
    let contract = index
        .items
        .iter()
        .find(|item| item.name == "Contract")
        .expect("contract");
    let syn::Item::Trait(contract) = &contract.syntax else {
        panic!("trait");
    };
    assert_eq!(contract.attrs.len(), 1);
    assert_eq!(contract.supertraits.len(), 1);
    assert_eq!(contract.items.len(), 1);
    assert_eq!(index.implementations.len(), 1);
    assert_eq!(index.implementations[0].syntax.items.len(), 1);
    assert!(
        index
            .items
            .iter()
            .any(|item| matches!(item.syntax, syn::Item::Type(_)))
    );
    assert!(
        index
            .items
            .iter()
            .any(|item| matches!(item.syntax, syn::Item::Fn(_)))
    );
    assert!(index.unanalysable.is_empty());
}

#[test]
fn given_qualified_self_super_and_glob_exports_when_resolved_then_all_routes_reach_the_same_definition()
 {
    let index = build_index(
        "mod model { pub struct Value; } mod export { pub use crate::model::*; } mod nested { mod child { fn call() { super::super::model::Value; crate::export::Value; } } }",
    );
    let direct = resolve(&index, 0, "self::model::Value");
    assert_eq!(direct, resolve(&index, 0, "crate::export::Value"));
    assert_eq!(direct, resolve(&index, 4, "super::super::model::Value"));
    assert!(index.unanalysable.is_empty());
}

#[test]
fn given_two_internal_globs_when_resolved_then_ambiguity_retains_both_definitions() {
    let index = build_index(
        "mod left { pub struct Value; } mod right { pub struct Value; } mod consumer { use crate::left::*; use crate::right::*; fn call() { Value; } }",
    );
    let Resolution::Internal(targets) = resolve(&index, 3, "Value") else {
        panic!("both definitions remain");
    };
    assert_eq!(targets.len(), 2);
    assert_ne!(targets[0].module, targets[1].module);
}

#[test]
fn given_eight_or_nine_reexports_when_resolved_then_only_the_documented_depth_is_followed() {
    let mut source = "mod origin { pub struct Value; }".to_owned();
    for depth in 1..=9 {
        let previous = if depth == 1 {
            "origin".to_owned()
        } else {
            format!("level{}", depth - 1)
        };
        source.push_str(&format!(
            "mod level{depth} {{ pub use crate::{previous}::Value; }}"
        ));
    }
    let index = build_index(&source);
    assert!(matches!(
        resolve(&index, 0, "crate::level8::Value"),
        Resolution::Internal(_)
    ));
    assert_eq!(
        resolve(&index, 0, "crate::level9::Value"),
        Resolution::Unresolved
    );
}

#[test]
fn given_generic_and_value_shadowing_when_indexed_then_type_names_keep_their_own_namespace() {
    let index = build_index(
        "mod model { pub struct Value; } use crate::model::Value; fn inspect<T>(input: T) { let Value = input; let typed: Option<Value> = None; Value; typed; <[_]>::is_empty; <Value>::new; } fn other() { let missing: T; }",
    );
    assert_eq!(index.unanalysable.len(), 1);
    assert_eq!(index.unanalysable[0].subject, "T");
    assert!(index.edges.iter().any(|edge| edge.path == ["Value"]));
}

#[test]
fn given_restricted_visibility_when_indexed_then_the_scope_path_is_neither_an_edge_nor_a_reference()
{
    let index = build_index(
        "mod flow { pub(crate) struct A; pub(in crate::flow) struct B; pub(super) struct C; pub(in crate::flow::inner) struct D; pub(crate) fn run() { crate::flow::A; } mod inner { pub(in crate::flow) struct E; } }",
    );
    assert!(index.unanalysable.is_empty());
    assert_eq!(1, index.edges.len());
    assert_eq!(index.edges[0].path, ["crate", "flow", "A"]);
    assert_eq!(1, index.references.len());
}

#[test]
fn given_cfg_import_alias_when_indexed_then_its_predicate_is_carried_by_consuming_edges() {
    let index = build_index(
        "mod model { pub struct Value; } #[cfg(feature = \"left\")] use crate::model::Value as Alias; #[cfg(not(feature = \"left\"))] fn call() { Alias; }",
    );
    let usage = index
        .edges
        .iter()
        .find(|edge| edge.path == ["Alias"])
        .expect("alias usage");
    assert!(!usage.cfg.is_satisfiable());
}

#[test]
fn given_block_imports_and_matches_bindings_when_indexed_then_names_have_their_lexical_scope() {
    let index = build_index(
        "mod model { pub struct Value; } fn inspect(input: Option<u32>) { { use crate::model::Value as Alias; let typed: Option<Alias> = None; typed; } matches!(input, Some(value) if value > 0); matches!(input, Some(value)); let missing: Alias; }",
    );
    assert_eq!(index.unanalysable.len(), 1);
    assert_eq!(index.unanalysable[0].subject, "Alias");
}

#[test]
fn given_expression_cfg_and_impl_method_cfg_when_indexed_then_every_edge_carries_the_condition() {
    let index = build_index(
        "mod model { pub struct Value; } fn call() { #[cfg(feature = \"left\")] crate::model::Value; } struct Holder; impl Holder { #[cfg(not(feature = \"left\"))] fn call() { crate::model::Value; } }",
    );
    let left = index
        .edges
        .iter()
        .find(|edge| edge.source == 0 && edge.path == ["crate", "model", "Value"])
        .expect("first edge");
    let right = index
        .edges
        .iter()
        .rev()
        .find(|edge| edge.source == 0 && edge.path == ["crate", "model", "Value"])
        .expect("second edge");
    assert!(!left.cfg.and(&right.cfg).is_satisfiable());
}
