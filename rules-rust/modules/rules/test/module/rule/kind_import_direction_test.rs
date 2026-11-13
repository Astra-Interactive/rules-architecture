use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::config::model::Role;
use crate::workspace::util::parse_toml_file;

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
fn given_restricted_visibility_in_platform_model_when_checked_then_the_scope_word_is_no_import() {
    for source in [
        "mod flow { pub(crate) mod model { pub(crate) struct SubscriptionId(pub(crate) u64); } }",
        "mod text { pub(crate) mod model { pub(crate) enum HostTextError { Rejected { reason: String } } } }",
        "mod flow { pub(crate) mod model { pub(in crate::flow) struct Scoped; pub(super) struct Parent; } }",
    ] {
        let context = read_import_context(source, Role::CorePlatform, &[], &[], "platform-plugin");
        let findings = KindImportDirection.check(&context);
        assert!(findings.is_empty(), "{source}: {findings:?}");
    }
}

#[test]
fn given_real_use_of_a_kindless_module_in_platform_model_when_checked_then_the_edge_stays_reported()
{
    for source in [
        "mod flow {\npub(crate) struct HandlerSubscription;\npub(crate) mod model {\nuse crate::flow::HandlerSubscription;\n}\n}",
        "mod flow {\npub(crate) struct HandlerSubscription;\npub(crate) mod model {\nfn call() { crate::flow::HandlerSubscription; }\n}\n}",
    ] {
        let context = read_import_context(source, Role::CorePlatform, &[], &[], "platform-plugin");
        let findings = KindImportDirection.check(&context);
        assert_eq!(1, findings.len(), "{source}: {findings:?}");
        assert!(
            findings[0].finding.message.contains("`flow`"),
            "{findings:?}"
        );
    }
}

#[test]
fn given_fake_of_a_di_trait_that_hands_out_other_fakes_when_checked_then_the_replaced_contract_and_the_fakes_are_allowed()
 {
    let context = read_import_context(
        "mod di { pub trait HostModule {} }\n#[cfg(any(test, feature = \"fake\"))]\npub mod fixture {\npub mod scheduling { pub mod fake { pub struct FakeScheduler; } }\npub mod host { pub mod fake {\nuse crate::di::HostModule;\nuse crate::fixture::scheduling::fake::FakeScheduler;\npub struct FakeHostModule { pub scheduler: std::sync::Arc<FakeScheduler> }\nimpl HostModule for FakeHostModule {}\n} }\n}",
        Role::CorePlatform,
        &[],
        &[],
        "platform-plugin",
    );
    let findings = KindImportDirection.check(&context);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn given_fake_that_uses_di_types_it_does_not_implement_or_src_implementations_when_checked_then_each_import_is_reported()
 {
    for source in [
        "mod di { pub trait OtherModule {} }\n#[cfg(any(test, feature = \"fake\"))]\npub mod fixture { pub mod host { pub mod fake {\nuse crate::di::OtherModule;\npub struct FakeHost;\n} } }",
        "mod di { pub struct CoreModule; }\n#[cfg(any(test, feature = \"fake\"))]\npub mod fixture { pub mod host { pub mod fake {\nuse crate::di::CoreModule;\npub struct FakeHost { pub core: std::sync::Arc<CoreModule> }\n} } }",
        "mod lifecycle { pub mod memory { pub struct LifecycleSwitch; } }\n#[cfg(test)]\nmod test { mod other { pub(crate) mod fake {\nuse crate::lifecycle::memory::LifecycleSwitch;\npub(crate) struct FakeUnloadingLifecycle { pub(crate) switch: std::sync::Arc<LifecycleSwitch> }\n} } }",
        "mod di { pub trait HostModule {} }\n#[cfg(any(test, feature = \"fake\"))]\npub mod fixture { pub mod host { pub mod fake {\nuse crate::di::HostModule;\npub struct FakeHost;\n} } }",
    ] {
        let context = read_import_context(source, Role::CorePlatform, &[], &[], "platform-plugin");
        let findings = KindImportDirection.check(&context);
        assert!(
            findings
                .iter()
                .any(|finding| finding.finding.location.line == Some(4)),
            "{source}: {findings:?}"
        );
    }
}

#[test]
fn given_test_tree_fake_when_it_uses_the_memory_implementation_of_its_own_concept_then_the_import_is_allowed()
 {
    for source in [
        "mod task { pub mod memory { pub struct TaskScope; } }\n#[cfg(test)]\nmod test { mod task { pub(crate) mod fake {\nuse crate::task::memory::TaskScope;\npub(crate) struct FakeAbortingJob { pub(crate) scope: std::sync::Arc<TaskScope> }\n} } }",
        "mod scheduling { pub mod task { pub mod memory { pub struct TaskScope; } } }\n#[cfg(test)]\nmod test { mod scheduling { mod task { pub(crate) mod fake {\nuse crate::scheduling::task::memory::TaskScope;\npub(crate) struct FakeAbortingJob { pub(crate) scope: std::sync::Arc<TaskScope> }\n} } } }",
        "#[cfg(test)]\nmod test { mod task { pub(crate) mod memory { pub(crate) struct InMemoryTaskStore; }\npub(crate) mod fake {\nuse crate::test::task::memory::InMemoryTaskStore;\npub(crate) struct FakeAbortingJob { pub(crate) store: std::sync::Arc<InMemoryTaskStore> }\n} } }",
    ] {
        let context = read_import_context(source, Role::CorePlatform, &[], &[], "platform-plugin");
        let findings = KindImportDirection.check(&context);
        assert!(findings.is_empty(), "{source}: {findings:?}");
    }
}

#[test]
fn given_fake_outside_the_test_tree_or_memory_of_another_concept_or_other_implementation_when_checked_then_the_import_is_reported()
 {
    for source in [
        "mod task { pub mod memory { pub struct TaskScope; } }\n#[cfg(any(test, feature = \"fake\"))]\npub mod fixture { pub mod task { pub mod fake {\nuse crate::task::memory::TaskScope;\npub struct FakeAbortingJob { pub scope: std::sync::Arc<TaskScope> }\n} } }",
        "mod task { pub mod memory { pub struct TaskScope; } pub mod fake {\nuse crate::task::memory::TaskScope;\npub struct FakeAbortingJob { pub scope: std::sync::Arc<TaskScope> }\n} }",
        "mod task { pub mod memory { pub struct TaskScope; } }\n#[cfg(test)]\nmod test { mod other { pub(crate) mod fake {\nuse crate::task::memory::TaskScope;\npub(crate) struct FakeAbortingJob { pub(crate) scope: std::sync::Arc<TaskScope> }\n} } }",
        "mod task { pub mod file { pub struct FileTaskStore; } }\n#[cfg(test)]\nmod test { mod task { pub(crate) mod fake {\nuse crate::task::file::FileTaskStore;\npub(crate) struct FakeAbortingJob { pub(crate) store: std::sync::Arc<FileTaskStore> }\n} } }",
        "mod scheduling { pub mod memory { pub struct TaskScope; } pub mod task {} }\n#[cfg(test)]\nmod test { mod scheduling { mod task { pub(crate) mod fake {\nuse crate::scheduling::memory::TaskScope;\npub(crate) struct FakeAbortingJob { pub(crate) scope: std::sync::Arc<TaskScope> }\n} } } }",
        "#[cfg(test)]\nmod test { mod task { pub(crate) mod memory { pub(crate) struct InMemoryTaskStore; } }\nmod other { pub(crate) mod fake {\nuse crate::test::task::memory::InMemoryTaskStore;\npub(crate) struct FakeAbortingJob { pub(crate) store: std::sync::Arc<InMemoryTaskStore> }\n} } }",
    ] {
        let context = read_import_context(source, Role::CorePlatform, &[], &[], "platform-plugin");
        let findings = KindImportDirection.check(&context);
        assert!(
            findings.iter().any(|finding| {
                finding.finding.location.line == Some(4) || finding.finding.location.line == Some(2)
            }),
            "{source}: {findings:?}"
        );
    }
}

#[test]
fn given_fake_implementing_another_trait_of_the_same_name_or_a_trait_of_another_kind_when_checked_then_the_import_is_reported()
 {
    for source in [
        "mod di { pub trait HostModule {} }\nmod api { pub trait HostModule {} }\n#[cfg(any(test, feature = \"fake\"))]\npub mod fixture { pub mod host { pub mod fake {\nuse crate::di::HostModule as DiHost;\npub struct FakeHost;\nimpl crate::api::HostModule for FakeHost {}\n} } }",
        "mod usecase { pub trait Runner {} }\n#[cfg(any(test, feature = \"fake\"))]\npub mod fixture { pub mod host { pub mod fake {\nuse crate::usecase::Runner;\npub struct FakeRunner;\nimpl Runner for FakeRunner {}\n} } }",
        "mod lifecycle { pub mod memory { pub trait Switch {} } }\n#[cfg(any(test, feature = \"fake\"))]\npub mod fixture { pub mod host { pub mod fake {\nuse crate::lifecycle::memory::Switch;\npub struct FakeSwitch;\nimpl Switch for FakeSwitch {}\n} } }",
    ] {
        let context = read_import_context(source, Role::CorePlatform, &[], &[], "platform-plugin");
        let findings = KindImportDirection.check(&context);
        let has_import_row = findings.iter().any(|finding| {
            finding.finding.location.line == Some(5) || finding.finding.location.line == Some(4)
        });
        assert!(has_import_row, "{source}: {findings:?}");
    }
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

#[test]
fn given_ui_entry_kind_importing_fixed_entry_kinds_when_checked_then_reports_each_fixed_kind() {
    let mut context = read_import_context(
        "mod billing { pub mod command { pub struct InvoiceCommand; } pub mod event { pub struct InvoiceListener; } }\nmod screen { pub mod menu { pub struct Menu; } pub mod slot {\nuse crate::billing::command::InvoiceCommand;\nuse crate::billing::event::InvoiceListener;\nuse crate::screen::menu::Menu;\n} }",
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
    let mut lines: Vec<Option<u32>> = KindImportDirection
        .check(&context)
        .iter()
        .map(|finding| finding.finding.location.line)
        .collect();
    lines.sort();
    assert_eq!(vec![Some(3), Some(4)], lines);
}

fn declare_role_kinds(context: &mut RuleContext, words: &str) {
    context.workspace.root_manifest = parse_toml_file(
        "Cargo.toml",
        &format!("[workspace.metadata.klibs-rules]\nproduct-kind=\"app\"\nrole-kinds=[{words}]\n"),
    )
    .expect("fixture metadata");
}

fn count_pair_findings(source: &str, target: &str, role: Role) -> usize {
    let source = format!(
        "mod left {{ mod {source} {{ fn call() {{ crate::right::{target}::Target; }} }} }} mod right {{ mod {target} {{ pub struct Target; }} }}"
    );
    let mut context = read_import_context(&source, role, &[], &[], "app");
    declare_role_kinds(&mut context, "\"limiter\", \"registry\"");
    KindImportDirection.check(&context).len()
}

#[test]
fn given_a_role_kind_as_source_when_checked_then_only_model_util_and_role_kinds_are_allowed() {
    for (target, expected) in [
        ("model", 0),
        ("util", 0),
        ("registry", 0),
        ("api", 1),
        ("usecase", 1),
        ("command", 1),
        ("file", 1),
        ("stub", 1),
        ("di", 1),
    ] {
        assert_eq!(
            count_pair_findings("limiter", target, Role::Context),
            expected,
            "limiter -> {target}"
        );
    }
}

#[test]
fn given_a_role_kind_as_target_when_checked_then_only_the_listed_kinds_may_use_it() {
    for (source, expected) in [
        ("usecase", 0),
        ("command", 0),
        ("file", 0),
        ("stub", 0),
        ("di", 0),
        ("registry", 0),
        ("model", 1),
        ("api", 1),
        ("util", 1),
        ("fake", 1),
    ] {
        assert_eq!(
            count_pair_findings(source, "limiter", Role::Context),
            expected,
            "{source} -> limiter"
        );
    }
}

#[test]
fn given_role_kinds_at_the_crate_root_and_in_a_technology_unit_when_checked_then_the_same_rows_hold()
 {
    let mut context = read_import_context(
        "mod limiter { pub struct VoteLimiter; } mod usecase { fn call() { crate::limiter::VoteLimiter; } } mod api { fn call() { crate::limiter::VoteLimiter; } }",
        Role::Context,
        &[],
        &[],
        "app",
    );
    declare_role_kinds(&mut context, "\"limiter\"");
    let findings = KindImportDirection.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].finding.subject, "api");
    let mut technology = read_import_context(
        "mod helper { fn call() { crate::limiter::VoteLimiter; } } mod limiter { pub struct VoteLimiter; }",
        Role::Technology,
        &[],
        &[],
        "app",
    );
    declare_role_kinds(&mut technology, "\"limiter\"");
    assert!(KindImportDirection.check(&technology).is_empty());
}

#[test]
fn given_role_kind_fixtures_when_checked_then_exact_wrong_imports_report_and_correct_ones_pass() {
    let context = read_case_context("KindImportDirection", "fail-role-kind");
    let mut actual: Vec<String> = KindImportDirection
        .check(&context)
        .iter()
        .map(|finding| {
            format!(
                "{} {}",
                finding.finding.location.line.expect("finding line"),
                finding.finding.subject
            )
        })
        .collect();
    actual.sort();
    assert_eq!(
        actual,
        ["10 util", "14 limiter", "15 limiter", "2 api", "6 model"]
    );
    let context = read_case_context("KindImportDirection", "pass-role-kind");
    assert!(KindImportDirection.check(&context).is_empty());
}

#[test]
fn given_util_using_api_items_when_checked_then_only_a_trait_is_reported() {
    for (source, declaration, role, expected) in [
        ("util", "pub trait Repo {}", Role::Context, 1),
        ("util", "pub trait Repo {}", Role::Technology, 1),
        ("util", "pub struct Repo;", Role::Context, 0),
        ("util", "pub enum Repo { Newest }", Role::Context, 0),
        ("usecase", "pub trait Repo {}", Role::Context, 0),
        ("fake", "pub trait Repo {}", Role::Context, 0),
    ] {
        let code = format!(
            "mod left {{ mod {source} {{ fn call() {{ crate::right::api::Repo; }} }} }} mod right {{ mod api {{ {declaration} }} }}"
        );
        let context = read_import_context(&code, role, &[], &[], "app");
        assert_eq!(
            KindImportDirection.check(&context).len(),
            expected,
            "{source} {declaration} {role:?}"
        );
    }
}

#[test]
fn given_util_trait_fixtures_when_checked_then_the_trait_import_and_use_report_and_data_passes() {
    let context = read_case_context("KindImportDirection", "fail-util-trait");
    let mut actual: Vec<String> = KindImportDirection
        .check(&context)
        .iter()
        .map(|finding| {
            format!(
                "{} {}",
                finding.finding.location.line.expect("finding line"),
                finding.finding.subject
            )
        })
        .collect();
    actual.sort();
    assert_eq!(actual, ["11 util", "12 util"]);
    let context = read_case_context("KindImportDirection", "pass-util-trait");
    assert!(KindImportDirection.check(&context).is_empty());
}
