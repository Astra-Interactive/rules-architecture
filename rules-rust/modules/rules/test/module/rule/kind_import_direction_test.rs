use crate::check::api::Rule;
use crate::config::model::Role;

#[test]
fn given_public_reexport_across_kinds_when_checked_then_only_the_kind_own_file_export_is_exempt() {
    let context = read_import_context(
        "mod model { pub(crate) use crate::api::Contract; } mod api { pub trait Contract {} }",
        Role::Context,
        &[],
        &[],
        "app",
    );
    assert_eq!(KindImportDirection.check(&context).len(), 1);
}
use crate::module::rule::KindImportDirection;
use crate::test::check::usecase::read_case_context;
use crate::test::module::util::assert_import_pair;
use crate::test::module::util::assert_resolution_cases;
use crate::test::module::util::read_import_context;
use crate::test::source::syn::build_index;

#[test]
fn given_a_unit_returning_business_contract_when_checked_then_registration_shape_alone_does_not_forbid_domain_use()
 {
    let context = read_import_context(
        "mod api { pub trait ClearCache { fn clear(&self); } } mod usecase { fn invoke(cache:&dyn crate::api::ClearCache) { cache.clear(); } }",
        Role::Context,
        &[],
        &[],
        "app",
    );
    let contract = context.indexes[0]
        .items
        .iter()
        .find(|item| item.name == "ClearCache")
        .unwrap();
    assert!(context.has_registration_shape(0, contract));
    assert!(KindImportDirection.check(&context).is_empty());
}

#[test]
fn given_a_known_framework_registration_trait_in_mixed_core_when_used_from_domain_then_workspace_target_is_reported()
 {
    let mut context = crate::test::source::util::read_resolution_context();
    context.project_config.platform_words = vec!["framework".to_owned()];
    for config in &mut context.project_config.crates {
        config.role = Some(match config.name.as_str() {
            "digest-package" => Role::Core,
            "framework-context" => Role::Platform,
            _ => Role::Impl,
        });
    }
    let findings = KindImportDirection.check(&context);
    assert_eq!(1, findings.len(), "{findings:?}");
    assert_eq!("Registrar", findings[0].finding.subject);
    assert_eq!("owner/src/lib.rs", findings[0].finding.location.path);
    assert_eq!(Some(42), findings[0].finding.location.line);
    assert!(
        matches!(&findings[0].anchor,crate::check::model::Anchor::Node{file,span} if file=="owner/src/lib.rs" && span.start.line==42)
    );
    let target = context
        .indexes
        .iter()
        .enumerate()
        .find_map(|(ordinal, index)| {
            index
                .items
                .iter()
                .find(|item| item.name == "Registrar")
                .map(|item| (ordinal, item))
        })
        .unwrap();
    assert!(context.is_platform_registration_trait(target.0, target.1));
}

#[test]
fn given_technology_concept_helpers_when_checked_then_implementation_access_does_not_leak_into_domain_models()
 {
    for (target, expected) in [("api", 0), ("usecase", 1)] {
        let source = format!(
            "mod helper {{ fn call() {{ crate::{target}::Target; }} }} mod {target} {{ pub struct Target; }}"
        );
        let context = read_import_context(&source, Role::Technology, &[], &[], "app");
        assert_eq!(KindImportDirection.check(&context).len(), expected);
    }
    let context = read_import_context(
        "mod representation { pub struct Row; } mod model { fn call() { crate::representation::Row; } }",
        Role::Technology,
        &[],
        &[],
        "app",
    );
    assert_eq!(KindImportDirection.check(&context).len(), 1);
}

#[test]
fn given_persistent_phase3_kind_pairs_when_checked_then_only_the_technology_role_allows_util_to_table()
 {
    assert_import_pair(
        &KindImportDirection,
        "KindImportDirection",
        Role::Technology,
    );
}

#[test]
fn given_inside_unit_kind_rows_when_checked_then_each_restricted_row_has_allowed_and_forbidden_edges()
 {
    for (source, allowed, forbidden) in [
        ("model", "model", "api"),
        ("api", "model", "api"),
        ("usecase", "api", "usecase"),
        ("util", "model", "usecase"),
        ("stub", "api", "di"),
        ("fake", "api", "usecase"),
        ("command", "usecase", "di"),
        ("warmup", "usecase", "di"),
        ("migration", "api", "di"),
        ("error", "model", "di"),
        ("file", "api", "usecase"),
    ] {
        for (target, expected) in [(allowed, 0), (forbidden, 1)] {
            let source = format!(
                "mod left {{ mod {source} {{ fn call() {{ crate::right::{target}::Target; }} }} }} mod right {{ mod {target} {{ pub struct Target; }} }}"
            );
            let context = read_import_context(&source, Role::Context, &[], &[], "app");
            assert_eq!(
                KindImportDirection.check(&context).len(),
                expected,
                "{source}"
            );
        }
    }
    for target in [
        "model", "api", "usecase", "util", "file", "command", "table", "entity", "database", "di",
    ] {
        let source = format!(
            "mod left {{ mod di {{ fn call() {{ crate::right::{target}::Target; }} }} }} mod right {{ mod {target} {{ pub struct Target; }} }}"
        );
        assert!(
            KindImportDirection
                .check(&read_import_context(
                    &source,
                    Role::Context,
                    &[],
                    &[],
                    "app"
                ))
                .is_empty()
        );
    }
}

#[test]
fn given_technology_unit_helpers_when_checked_then_util_can_use_technology_data_but_domain_models_remain_restricted()
 {
    for role in [Role::Technology, Role::Platform, Role::CorePlatform] {
        for target in ["table", "entity", "database"] {
            let source = format!(
                "mod util {{ fn call() {{ crate::{target}::Target; }} }} mod {target} {{ pub struct Target; }}"
            );
            assert!(
                KindImportDirection
                    .check(&read_import_context(&source, role, &[], &[], "app"))
                    .is_empty(),
                "{role:?}/{target}"
            );
            assert_eq!(
                KindImportDirection
                    .check(&read_import_context(
                        &source,
                        Role::Context,
                        &[],
                        &[],
                        "app"
                    ))
                    .len(),
                1
            );
        }
    }
    let context = read_import_context(
        "mod model { fn call() { crate::table::Row; } } mod table { pub struct Row; }",
        Role::Technology,
        &[],
        &[],
        "app",
    );
    assert_eq!(KindImportDirection.check(&context).len(), 1);
    let context = read_import_context(
        "mod file { mod table { pub struct Row; } fn call() { self::table::Row; } }",
        Role::Context,
        &[],
        &[],
        "app",
    );
    assert!(KindImportDirection.check(&context).is_empty());
}

#[test]
fn given_entry_point_root_paths_when_checked_then_only_di_and_hosted_entry_kinds_are_allowed() {
    let mut context = read_case_context("ImportOrdering", "pass-syntax");
    context.indexes = vec![build_index(
        "mod model { pub struct Value; } mod di { pub struct Root; } mod command { pub struct Command; } fn main() { crate::model::Value; crate::di::Root; crate::command::Command; }",
    )];
    context.indexes[0].crate_name = context.project_config.crates[0].name.clone();
    context.project_config.crates[0].role = Some(crate::config::model::Role::EntryPoint);
    let findings = KindImportDirection.check(&context);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].finding.message.contains("model"));
}

#[test]
fn given_persistent_resolution_cases_when_checked_then_model_to_api_import_is_the_only_rule_finding()
 {
    assert_resolution_cases(
        &KindImportDirection,
        "KindImportDirection",
        "modules/sample/src/lib.rs:6\tmodel\n",
    );
}

#[test]
fn given_kind_edges_when_checked_then_forbidden_edges_are_reported_and_test_edges_are_free() {
    let mut context = read_case_context("ImportOrdering", "pass-syntax");
    context.indexes = vec![build_index(
        "mod model { pub struct Value; fn call() { crate::api::Contract; } } mod api { pub struct Contract; } mod util { fn call() { crate::api::Contract; crate::model::Value; } } #[cfg(test)] mod test { mod model { fn check() { crate::api::Contract; } } }",
    )];
    let findings = KindImportDirection.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.subject, "model");
}

#[test]
fn given_usecase_and_di_edges_when_checked_then_only_cross_usecase_and_non_di_access_to_di_are_reported()
 {
    let mut context = read_case_context("ImportOrdering", "pass-syntax");
    context.indexes = vec![build_index(
        "mod first { pub mod usecase { pub struct First; fn call() { crate::second::usecase::Second; } } } mod second { pub mod usecase { pub struct Second; } } mod di { pub struct Root; fn call() { crate::first::usecase::First; } } mod util { fn call() { crate::di::Root; } }",
    )];
    assert_eq!(KindImportDirection.check(&context).len(), 2);
}

#[test]
fn given_factory_runtime_contract_parameters_when_checked_then_its_import_and_parameter_are_allowed()
 {
    let mut context = read_case_context("ImportOrdering", "pass-syntax");
    context.indexes = vec![build_index(
        "mod other { pub mod api { pub trait Other {} } } mod api { use crate::other::api::Other; pub trait Product {} pub trait ProductFactory { fn invoke(other: Box<dyn Other>) -> Box<dyn Product>; } }",
    )];
    assert!(KindImportDirection.check(&context).is_empty());
    context.indexes = vec![build_index(
        "mod other { pub mod api { pub trait Other {} } } mod api { use crate::other::api::Other; pub trait Product {} pub trait ProductFactory { fn invoke(other: Box<dyn Other>) -> bool; } }",
    )];
    assert_eq!(KindImportDirection.check(&context).len(), 2);
}

#[test]
fn given_fake_in_fixture_tree_when_it_uses_a_file_implementation_then_only_that_import_is_reported()
{
    let context = read_import_context(
        "mod invoice { pub mod api { pub trait InvoiceRepository {} } pub mod file { pub struct FileInvoiceRepository; } }\n#[cfg(any(test, feature = \"fake\"))] pub mod fixture { pub mod invoice { pub mod fake {\nuse crate::invoice::api::InvoiceRepository;\nuse crate::invoice::file::FileInvoiceRepository;\n} } }",
        Role::Context,
        &[],
        &[],
        "app",
    );
    let findings = KindImportDirection.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!("KindImportDirection", findings[0].finding.rule_id.0);
    assert_eq!(Some(4), findings[0].finding.location.line);
    assert!(findings[0].finding.message.contains("invoice::file"));
}

#[test]
fn given_entry_point_kinds_when_checked_then_only_ui_kinds_may_use_each_other() {
    let mut context = read_import_context(
        "mod billing { pub mod job { pub struct InvoiceJob; } pub mod command {\nuse crate::billing::job::InvoiceJob;\n} }\nmod task { pub mod service { pub struct TaskScope; } }\nmod report { pub mod service {\nuse crate::task::service::TaskScope;\n} }\nmod screen { pub mod menu { pub struct Menu; } pub mod slot {\nuse crate::screen::menu::Menu;\n} }",
        Role::Context,
        &[],
        &[],
        "app",
    );
    context.workspace.root_manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[workspace.metadata.klibs-rules]\nproduct-kind=\"app\"\nentry-point-words=[\"menu\",\"slot\"]\n",
    )
    .expect("fixture metadata");
    let findings = KindImportDirection.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!("KindImportDirection", findings[0].finding.rule_id.0);
    assert_eq!(Some(2), findings[0].finding.location.line);
    assert!(findings[0].finding.message.contains("billing::job"));
}
