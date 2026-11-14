use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::config::model::Role;
use crate::module::rule::KindShape;
use crate::test::check::usecase::read_case_context;
use crate::test::module::util::read_shape_context;
use crate::test::module::util::read_workspace_shape_context_with_files;
use crate::workspace::util::parse_toml_file;

fn list_findings_with(context: &RuleContext, needle: &str) -> Vec<String> {
    let mut lines: Vec<String> = KindShape
        .check(context)
        .iter()
        .filter(|finding| finding.finding.message.contains(needle))
        .map(|finding| format!("{}\t{}", finding.finding.subject, finding.finding.message))
        .collect();
    lines.sort();
    lines
}

#[test]
fn given_empty_kind_block_mirrored_in_the_test_and_fixture_trees_when_checked_then_only_the_source_reports()
 {
    for mirror in [
        "#[cfg(test)]\n#[path = \"../test\"]\nmod test { mod order { mod model {} } }",
        "#[cfg(any(test, feature = \"fake\"))]\n#[path = \"../fixture\"]\npub mod fixture { mod order { mod model {} } }",
    ] {
        let context = read_shape_context(&format!("mod order {{ mod model {{}} }}\n{mirror}"), &[]);
        let lines = list_findings_with(&context, "never empty");
        assert_eq!(1, lines.len(), "{lines:#?}");
        assert!(
            lines[0].starts_with("case_sample::order::model\t"),
            "{lines:#?}"
        );
    }
}

#[test]
fn given_empty_kind_block_only_in_the_test_tree_when_checked_then_it_reports_as_before() {
    let context = read_shape_context(
        "mod order { mod model { struct Invoice; } }\n#[cfg(test)]\n#[path = \"../test\"]\nmod test { mod order { mod model {} } }",
        &[],
    );
    let lines = list_findings_with(&context, "never empty");
    assert_eq!(1, lines.len(), "{lines:#?}");
    assert!(
        lines[0].starts_with("case_sample::test::order::model\t"),
        "{lines:#?}"
    );
}

#[test]
fn given_kind_block_mixing_files_and_blocks_in_the_source_and_the_test_tree_when_checked_then_only_the_source_reports()
 {
    let context = crate::test::module::util::read_placement_context(&[
        (
            "src/lib.rs",
            "mod settings {\n    mod model {\n        mod first;\n        mod second {}\n    }\n}\n#[cfg(test)]\n#[path = \"../test\"]\nmod test {\n    mod settings {\n        mod model {\n            mod first_test;\n            mod second {}\n        }\n    }\n}",
        ),
        ("src/settings/model/first.rs", "pub struct First;"),
        (
            "test/settings/model/first_test.rs",
            "#[test]\nfn given_first_when_built_then_ready() {}",
        ),
    ]);
    let lines = list_findings_with(&context, "does not mix file modules and inline blocks");
    assert_eq!(1, lines.len(), "{lines:#?}");
    assert!(lines[0].contains("::settings::model\t"), "{lines:#?}");
    assert!(!lines[0].contains("::test::"), "{lines:#?}");
}

#[test]
fn given_technology_concept_implementations_and_private_helpers_when_checked_then_only_unowned_helper_reports()
 {
    let mut context = read_shape_context(
        "mod invoice { mod repository; mod refreshing_cache; mod unrelated; mod model { struct Invoice; } mod api { trait InvoiceReader {} } }",
        &[
            (
                "invoice/repository.rs",
                "struct InvoiceStore { cache: super::refreshing_cache::RefreshingCache } impl super::api::InvoiceReader for InvoiceStore {}",
            ),
            (
                "invoice/refreshing_cache.rs",
                "pub(crate) struct RefreshingCache;",
            ),
            ("invoice/unrelated.rs", "struct Unrelated;"),
        ],
    );
    context.project_config.crates[0].role = Some(crate::config::model::Role::Technology);
    let findings = KindShape.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings[0].finding.subject,
        "case_sample::invoice::unrelated"
    );
}

const MIRROR_IMPLEMENTATION: &str =
    "pub(crate) struct CommandSlot; impl crate::slot::api::HandlerSlot for CommandSlot {}";

const SLOT_API: &str = "pub(crate) mod api { pub(crate) trait HandlerSlot {} }";

fn list_shape_subjects(source: &str, files: &[(&str, &str)], role: Role) -> Vec<String> {
    let mut context = read_shape_context(source, files);
    context.project_config.crates[0].role = Some(role);
    let mut subjects: Vec<String> = KindShape
        .check(&context)
        .iter()
        .map(|finding| finding.finding.subject.clone())
        .collect();
    subjects.sort();
    subjects
}

#[test]
fn given_test_mirror_of_an_accepted_concept_file_when_checked_then_the_mirror_is_judged_by_its_source()
 {
    let subjects = list_shape_subjects(
        &format!(
            "mod slot {{ mod command_slot; {SLOT_API} }}\n#[cfg(test)]\n#[path = \"../test\"]\nmod test {{ mod slot {{ mod command_slot_test; }} }}"
        ),
        &[
            ("slot/command_slot.rs", MIRROR_IMPLEMENTATION),
            (
                "../test/slot/command_slot_test.rs",
                "#[test] fn given_slot_when_run_then_ok() {}",
            ),
        ],
        Role::CorePlatform,
    );
    assert!(subjects.is_empty(), "{subjects:?}");
}

#[test]
fn given_fixture_file_directly_in_a_concept_when_checked_then_it_is_reported_even_beside_a_same_named_source()
 {
    let subjects = list_shape_subjects(
        &format!(
            "mod slot {{ mod command_slot; {SLOT_API} }}\n#[cfg(any(test, feature = \"fake\"))]\n#[path = \"../fixture\"]\npub mod fixture {{ pub mod slot {{ mod command_slot; }} }}"
        ),
        &[
            ("slot/command_slot.rs", MIRROR_IMPLEMENTATION),
            ("../fixture/slot/command_slot.rs", "pub struct SlotFixture;"),
        ],
        Role::CorePlatform,
    );
    assert_eq!(subjects, ["case_sample::fixture::slot::command_slot"]);
}

fn read_published_mirror_context(source: &str, files: &[(&str, &str)]) -> RuleContext {
    use crate::source::api::SourceReader;
    use crate::workspace::api::WorkspaceReader;
    let mut context = read_shape_context("", &[]);
    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path();
    std::fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nresolver = \"3\"\nmembers = [\"sample\"]\n[workspace.metadata.klibs-rules]\nproduct-kind = \"app\"\n",
    )
    .expect("workspace manifest");
    std::fs::create_dir_all(root.join("sample/src")).expect("source directory");
    std::fs::write(
        root.join("sample/Cargo.toml"),
        "[package]\nname = \"sample\"\nversion = \"0.1.0\"\nedition = \"2024\"\n[package.metadata.klibs-rules]\nrole = \"technology\"\n",
    )
    .expect("member manifest");
    std::fs::write(
        root.join("sample/src/lib.rs"),
        format!("mod slot {{ mod command_slot; mod loose; {SLOT_API} }}"),
    )
    .expect("root source");
    std::fs::create_dir_all(root.join("sample/src/slot")).expect("slot directory");
    std::fs::write(
        root.join("sample/src/slot/command_slot.rs"),
        MIRROR_IMPLEMENTATION,
    )
    .expect("source file");
    std::fs::write(root.join("sample/src/slot/loose.rs"), "struct Loose;").expect("loose file");
    std::fs::create_dir_all(root.join("sample/tests/it/slot")).expect("tests directory");
    std::fs::write(root.join("sample/tests/it/main.rs"), source).expect("tests root");
    for (path, text) in files {
        std::fs::write(root.join("sample/tests/it").join(path), text).expect("test file");
    }
    context.workspace = crate::workspace::cargo::CargoWorkspaceReader::new(
        crate::workspace::util::resolve_cargo_path(),
    )
    .read_workspace(root)
    .expect("read workspace");
    context.project_config =
        crate::config::util::parse_project_config(&context.workspace).expect("configuration");
    context.crates = crate::source::syn::SynSourceReader.read_sources(&context.workspace);
    context.indexes =
        crate::source::syn::CrateIndexBuilder::build_all(&context.crates, &context.workspace);
    context
}

#[test]
fn given_tests_it_mirror_of_an_accepted_concept_file_when_checked_then_the_source_twin_is_found_in_the_library_index()
 {
    let context = read_published_mirror_context(
        "mod slot { mod command_slot_test; mod loose_test; }",
        &[
            (
                "slot/command_slot_test.rs",
                "#[test] fn given_a_when_b_then_c() {}",
            ),
            (
                "slot/loose_test.rs",
                "#[test] fn given_a_when_b_then_c() {}",
            ),
        ],
    );
    let mut subjects: Vec<String> = KindShape
        .check(&context)
        .iter()
        .map(|finding| finding.finding.subject.clone())
        .collect();
    subjects.sort();
    assert_eq!(
        subjects,
        ["sample::slot::loose", "sample::slot::loose_test"]
    );
}

#[test]
fn given_test_mirror_of_a_rejected_or_missing_source_when_checked_then_the_mirror_is_reported_too()
{
    let subjects = list_shape_subjects(
        &format!(
            "mod slot {{ mod command_slot; mod loose; {SLOT_API} }}\n#[cfg(test)]\n#[path = \"../test\"]\nmod test {{ mod slot {{ mod command_slot_test; mod loose_test; mod orphan_test; }} }}"
        ),
        &[
            ("slot/command_slot.rs", MIRROR_IMPLEMENTATION),
            ("slot/loose.rs", "struct Loose;"),
            (
                "../test/slot/command_slot_test.rs",
                "#[test] fn given_a_when_b_then_c() {}",
            ),
            (
                "../test/slot/loose_test.rs",
                "#[test] fn given_a_when_b_then_c() {}",
            ),
            (
                "../test/slot/orphan_test.rs",
                "#[test] fn given_a_when_b_then_c() {}",
            ),
        ],
        Role::CorePlatform,
    );
    assert_eq!(
        subjects,
        [
            "case_sample::slot::loose",
            "case_sample::test::slot::loose_test",
            "case_sample::test::slot::orphan_test"
        ]
    );
}

#[test]
fn given_test_mirror_in_a_crate_that_is_not_a_mixed_unit_when_checked_then_source_and_mirror_report()
 {
    let subjects = list_shape_subjects(
        "mod slot { mod command_slot; }\n#[cfg(test)]\n#[path = \"../test\"]\nmod test { mod slot { mod command_slot_test; } }",
        &[
            ("slot/command_slot.rs", MIRROR_IMPLEMENTATION),
            (
                "../test/slot/command_slot_test.rs",
                "#[test] fn given_a_when_b_then_c() {}",
            ),
        ],
        Role::Api,
    );
    assert_eq!(
        subjects,
        [
            "case_sample::slot::command_slot",
            "case_sample::test::slot::command_slot_test"
        ]
    );
}

#[test]
fn given_app_api_fixture_fake_kind_when_checked_then_production_kind_whitelist_does_not_reject_fixture()
 {
    let mut context = read_shape_context(
        "#[cfg(any(test, feature = \"fake\"))] #[path = \"../fixture\"] pub mod fixture { pub mod invoice { pub mod fake { mod invoice_fake; } } }",
        &[(
            "../fixture/invoice/fake/invoice_fake.rs",
            "pub struct InvoiceFake;",
        )],
    );
    context.project_config.crates[0].role = Some(crate::config::model::Role::Api);
    assert!(KindShape.check(&context).is_empty());
}

#[test]
fn given_phase3_role_fixtures_when_checked_then_exact_wrong_sites_report_and_correct_roles_pass() {
    let context = crate::test::check::usecase::read_case_context("KindShape", "fail-phase3-role");
    let mut actual: Vec<String> = KindShape
        .check(&context)
        .iter()
        .map(|finding| {
            format!(
                "{}:{} {}",
                finding.finding.location.path,
                finding.finding.location.line.unwrap_or(0),
                finding.finding.subject
            )
        })
        .collect();
    actual.sort();
    assert_eq!(actual, ["modules/sample/src/lib.rs:1 case_sample::command"]);
    let context = crate::test::check::usecase::read_case_context("KindShape", "pass-phase3-role");
    assert!(KindShape.check(&context).is_empty());
}

#[test]
fn given_nested_kinds_and_empty_concepts_when_checked_then_only_nested_kind_is_reported() {
    let context = read_shape_context(
        "mod invoice {} mod model { mod util { fn compute_value() {} } }",
        &[],
    );
    let findings = KindShape.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.subject, "case_sample::model::util");
}

#[test]
fn given_declaration_shapes_when_checked_then_exact_wrong_sites_report_and_correct_shapes_pass() {
    let context = read_case_context("KindShape", "fail-declaration-shape");
    let mut actual: Vec<String> = KindShape
        .check(&context)
        .iter()
        .map(|finding| {
            format!(
                "{}:{} {}",
                finding.finding.location.path,
                finding.finding.location.line.expect("finding line"),
                finding.finding.subject
            )
        })
        .collect();
    actual.sort();
    assert_eq!(
        actual,
        [
            "modules/sample/src/lib.rs:1 case_sample::invoice",
            "modules/sample/src/lib.rs:2 case_sample::invoice::loose",
            "modules/sample/src/lib.rs:3 case_sample::invoice::model"
        ]
    );
    let context = read_case_context("KindShape", "pass-declaration-shape");
    assert!(KindShape.check(&context).is_empty());
}

#[test]
fn given_watch_sender_aliases_when_kind_checked_then_entry_point_ownership_is_preserved() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::KindShape,
        "mod model { type Updates = tokio::sync::watch::Sender<u64>;\nstruct State { updates: Updates } }",
        &[2],
    );
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::KindShape,
        "mod local { pub struct Sender; } mod model { struct State { updates: crate::local::Sender } }",
        &[],
    );
}

#[test]
fn given_a_third_concept_level_when_checked_then_only_the_third_level_reports() {
    let context = read_shape_context(
        "pub(crate) mod billing {\n    pub(crate) mod invoice {\n        pub(crate) mod line {\n            pub(crate) mod model {\n                pub(crate) use invoice_line::InvoiceLine;\n                mod invoice_line;\n            }\n        }\n    }\n}\npub(crate) mod shipping {\n    pub(crate) mod parcel {\n        pub(crate) mod model {\n            pub(crate) use parcel_record::ParcelRecord;\n            mod parcel_record;\n        }\n    }\n}",
        &[
            (
                "billing/invoice/line/model/invoice_line.rs",
                "pub(crate) struct InvoiceLine;",
            ),
            (
                "shipping/parcel/model/parcel_record.rs",
                "pub(crate) struct ParcelRecord;",
            ),
        ],
    );
    let findings = KindShape.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!("KindShape", findings[0].finding.rule_id.0);
    assert_eq!(
        findings[0].finding.subject,
        "case_sample::billing::invoice::line"
    );
    assert_eq!(Some(3), findings[0].finding.location.line);
}

#[test]
fn given_state_holder_interface_in_app_api_component_when_checked_then_only_the_struct_in_that_kind_reports()
 {
    let mut context = read_shape_context(
        "mod offer { pub mod component { pub trait OfferComponent: Send + Sync { fn offer_count(&self) -> u32; } } }\nmod stock { pub mod component { pub struct StockComponent; } }",
        &[],
    );
    context.project_config.crates[0].role = Some(crate::config::model::Role::Api);
    context.workspace.root_manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[workspace.metadata.klibs-rules]\nproduct-kind=\"app\"\nentry-point-words=[\"component\"]\n",
    )
    .expect("manifest");
    let findings = KindShape.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!("KindShape", findings[0].finding.rule_id.0);
    assert_eq!(findings[0].finding.subject, "case_sample::stock::component");
    assert_eq!(Some(2), findings[0].finding.location.line);
}

#[test]
fn given_cfg_specific_field_alias_when_checked_then_incompatible_watch_sender_alias_is_not_used() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::KindShape,
        "mod model { #[cfg(unix)] type Updates = tokio::sync::watch::Sender<u64>; #[cfg(not(unix))] type Updates = String; struct State { #[cfg(not(unix))] updates: Updates } }",
        &[],
    );
}

fn declare_role_kinds(context: &mut RuleContext, words: &str) {
    context.workspace.root_manifest = parse_toml_file(
        "Cargo.toml",
        &format!("[workspace.metadata.klibs-rules]\nproduct-kind=\"app\"\nrole-kinds=[{words}]\n"),
    )
    .expect("manifest");
}

fn list_findings(findings: &[RuleFinding]) -> Vec<String> {
    let mut lines: Vec<String> = findings
        .iter()
        .map(|finding| {
            format!(
                "{}:{} {}",
                finding.finding.location.path,
                finding.finding.location.line.expect("finding line"),
                finding.finding.subject
            )
        })
        .collect();
    lines.sort();
    lines
}

#[test]
fn given_role_kind_modules_in_a_concept_and_at_the_crate_root_when_checked_then_they_are_kinds() {
    let mut context = read_shape_context(
        "pub(crate) mod vote {\n    pub(crate) mod limiter {\n        pub(crate) use vote_cooldown_limiter::VoteCooldownLimiter;\n        mod vote_cooldown_limiter;\n    }\n}\npub(crate) mod registry {\n    pub(crate) use packet_registry::PacketRegistry;\n    mod packet_registry;\n}",
        &[
            (
                "vote/limiter/vote_cooldown_limiter.rs",
                "pub(crate) struct VoteCooldownLimiter;\nimpl VoteCooldownLimiter { fn new() -> Self { Self } }",
            ),
            (
                "registry/packet_registry.rs",
                "pub(crate) struct PacketRegistry;",
            ),
        ],
    );
    declare_role_kinds(&mut context, "\"limiter\", \"registry\"");
    let limiter = context.indexes[0]
        .modules
        .iter()
        .find(|module| {
            module
                .module
                .path
                .last()
                .is_some_and(|name| name == "limiter")
        })
        .expect("limiter module")
        .id;
    assert_eq!(Some(limiter), context.find_kind_module(0, limiter));
    let findings = KindShape.check(&context);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn given_role_kind_types_when_checked_then_only_names_without_the_word_report() {
    let mut context = read_shape_context(
        "mod limiter { mod vote_cooldown; mod window_kind; mod slot_limiter; }",
        &[
            (
                "limiter/vote_cooldown.rs",
                "pub(crate) struct VoteCooldown;",
            ),
            (
                "limiter/window_kind.rs",
                "pub(crate) enum WindowKind { Short }",
            ),
            (
                "limiter/slot_limiter.rs",
                "pub(crate) union SlotLimiter { raw: u32 }",
            ),
        ],
    );
    declare_role_kinds(&mut context, "\"limiter\"");
    assert_eq!(
        list_findings(&KindShape.check(&context)),
        [
            "limiter/vote_cooldown.rs:1 case_sample::limiter::VoteCooldown",
            "limiter/window_kind.rs:1 case_sample::limiter::WindowKind",
        ]
        .map(|line| format!("modules/sample/src/{line}"))
    );
}

#[test]
fn given_a_role_kind_word_with_underscores_when_checked_then_the_name_ends_with_its_pascal_case() {
    let mut context = read_shape_context(
        "mod rate_limiter { mod vote_rate_limiter; mod vote_limiter; }",
        &[
            (
                "rate_limiter/vote_rate_limiter.rs",
                "pub(crate) struct VoteRateLimiter;",
            ),
            (
                "rate_limiter/vote_limiter.rs",
                "pub(crate) struct VoteLimiter;",
            ),
        ],
    );
    declare_role_kinds(&mut context, "\"rate_limiter\"");
    assert_eq!(
        list_findings(&KindShape.check(&context)),
        [
            "modules/sample/src/rate_limiter/vote_limiter.rs:1 case_sample::rate_limiter::VoteLimiter"
        ]
    );
}

#[test]
fn given_a_role_kind_module_with_non_type_items_when_checked_then_each_is_reported() {
    let mut context = read_shape_context(
        "mod limiter { mod limit_items; }",
        &[(
            "limiter/limit_items.rs",
            "pub(crate) struct VoteLimiter;\n\
             pub(crate) trait Window {}\n\
             pub(crate) static WINDOW: u32 = 1;\n\
             pub(crate) const LIMIT: u32 = 1;\n\
             pub(crate) fn compute_window() -> u32 { 1 }\n\
             impl VoteLimiter { const MAX: u32 = 1; fn new() -> Self { Self } }\n\
             impl std::fmt::Debug for VoteLimiter {}",
        )],
    );
    declare_role_kinds(&mut context, "\"limiter\"");
    assert_eq!(
        list_findings(&KindShape.check(&context)),
        [
            "2 case_sample::limiter::Window",
            "3 case_sample::limiter::WINDOW",
            "4 case_sample::limiter::LIMIT",
            "5 case_sample::limiter::compute_window",
        ]
        .map(|line| format!("modules/sample/src/limiter/limit_items.rs:{line}"))
    );
}

#[test]
fn given_a_role_kind_module_with_macros_extern_items_and_aliases_when_checked_then_every_non_type_item_is_reported()
 {
    let mut context = read_shape_context(
        "mod limiter {\nextern crate core;\nmod limit_items;\n}\n#[cfg(test)]\nmod test {\nmod limiter {\nmod test_items;\n}\n}",
        &[
            (
                "limiter/limit_items.rs",
                "extern crate alloc;\n\
                 use std::fmt::Debug;\n\
                 macro_rules! window { () => { 60 } }\n\
                 build_table!();\n\
                 extern \"C\" { fn abs(input: i32) -> i32; }\n\
                 pub(crate) type Window = u32;\n\
                 pub(crate) struct VoteLimiter;\n\
                 impl VoteLimiter { fn new() -> Self { Self } }\n\
                 trait WindowAlias = Debug;",
            ),
            ("test/limiter/test_items.rs", "extern crate alloc;"),
        ],
    );
    declare_role_kinds(&mut context, "\"limiter\"");
    assert_eq!(
        list_findings(&KindShape.check(&context)),
        [
            "modules/sample/src/lib.rs:2 case_sample::limiter::core",
            "modules/sample/src/limiter/limit_items.rs:1 case_sample::limiter::alloc",
            "modules/sample/src/limiter/limit_items.rs:3 case_sample::limiter::window",
            "modules/sample/src/limiter/limit_items.rs:4 case_sample::limiter::build_table!",
            "modules/sample/src/limiter/limit_items.rs:5 case_sample::limiter::extern block",
            "modules/sample/src/limiter/limit_items.rs:9 case_sample::limiter::WindowAlias",
        ]
    );
}

#[test]
fn given_a_module_named_by_an_unconfigured_word_when_checked_then_it_reports_as_before() {
    let source = "pub(crate) mod vote { pub(crate) mod parser { mod ballot_parser; } }";
    let files = [(
        "vote/parser/ballot_parser.rs",
        "pub(crate) struct BallotParser;",
    )];
    let before = read_shape_context(source, &files);
    let mut configured = read_shape_context(source, &files);
    declare_role_kinds(&mut configured, "\"limiter\"");
    let reported = list_findings(&KindShape.check(&before));
    assert!(!reported.is_empty());
    assert!(KindShape.check(&before).iter().any(|finding| {
        finding
            .finding
            .message
            .contains("a concept holds kind blocks")
    }));
    assert_eq!(reported, list_findings(&KindShape.check(&configured)));
}

#[test]
fn given_a_role_kind_module_in_an_app_api_crate_when_checked_then_the_crate_role_reports_it() {
    let mut context = read_shape_context(
        "mod limiter { mod vote_limiter; }",
        &[("limiter/vote_limiter.rs", "pub(crate) struct VoteLimiter;")],
    );
    declare_role_kinds(&mut context, "\"limiter\"");
    assert!(KindShape.check(&context).is_empty());
    context.project_config.crates[0].role = Some(crate::config::model::Role::Api);
    let findings = KindShape.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].finding.subject, "case_sample::limiter");
    assert!(
        findings[0]
            .finding
            .message
            .contains("the crate role does not permit this kind")
    );
}

#[test]
fn given_technology_concept_file_with_only_a_standard_trait_impl_when_checked_then_it_is_reported()
{
    for implementation in [
        "impl Drop for Guard { fn drop(&mut self) {} }",
        "impl Default for Guard { fn default() -> Self { Self } }",
        "impl std::fmt::Display for Guard {}",
    ] {
        let subjects = list_shape_subjects(
            "mod invoice { mod guard; mod model { struct Invoice; } }",
            &[(
                "invoice/guard.rs",
                &format!("pub(crate) struct Guard; {implementation}"),
            )],
            Role::Technology,
        );
        assert_eq!(
            subjects,
            ["case_sample::invoice::guard"],
            "{implementation}"
        );
    }
}

#[test]
fn given_concept_file_with_only_a_contract_impl_for_a_foreign_type_when_checked_then_it_is_an_implementation()
 {
    let files = [(
        "invoice/inventory_invoice_reader.rs",
        "use futures_executor::Inventory;\nimpl super::api::InvoiceReader for Inventory {}",
    )];
    let root = "mod invoice { mod inventory_invoice_reader; mod api { trait InvoiceReader {} } }";
    assert!(list_shape_subjects(root, &files, Role::Technology).is_empty());
    assert!(list_shape_subjects(root, &files, Role::CorePlatform).is_empty());
    let standard_trait = [(
        "invoice/inventory_invoice_reader.rs",
        "use futures_executor::Inventory;\nimpl std::fmt::Display for Inventory {}",
    )];
    assert_eq!(
        list_shape_subjects(root, &standard_trait, Role::Technology),
        ["case_sample::invoice::inventory_invoice_reader"]
    );
}

#[test]
fn given_drop_guard_used_by_a_contract_implementation_of_its_concept_when_checked_then_it_is_accepted()
 {
    let subjects = list_shape_subjects(
        "mod invoice { mod repository; mod guard; mod api { trait InvoiceReader {} } }",
        &[
            (
                "invoice/repository.rs",
                "struct InvoiceStore { guard: super::guard::Guard } impl super::api::InvoiceReader for InvoiceStore {}",
            ),
            (
                "invoice/guard.rs",
                "pub(crate) struct Guard; impl Drop for Guard { fn drop(&mut self) {} }",
            ),
        ],
        Role::Technology,
    );
    assert!(subjects.is_empty(), "{subjects:?}");
}

fn list_platform_subjects(implementation: &str) -> Vec<String> {
    let mut context = read_shape_context(
        "mod bridge { mod host_bridge; }",
        &[(
            "bridge/host_bridge.rs",
            &format!("pub(crate) struct HostBridge; {implementation}"),
        )],
    );
    context.project_config.crates[0].role = Some(Role::Platform);
    context.project_config.platform_words = vec!["vendor".to_owned()];
    for index in &mut context.indexes {
        index
            .dependencies
            .extend(["vendor", "serde"].map(str::to_owned));
    }
    KindShape
        .check(&context)
        .iter()
        .map(|finding| finding.finding.subject.clone())
        .collect()
}

#[test]
fn given_impl_of_a_platform_library_trait_in_a_platform_crate_when_checked_then_the_file_is_an_implementation()
 {
    assert!(list_platform_subjects("impl vendor::Handler for HostBridge {}").is_empty());
    assert_eq!(
        list_platform_subjects("impl serde::Serialize for HostBridge {}"),
        ["case_sample::bridge::host_bridge"]
    );
}

#[test]
fn given_impl_of_an_api_trait_of_another_workspace_crate_when_checked_then_the_file_is_an_implementation()
 {
    let context = read_workspace_shape_context_with_files(
        &[
            (
                "shop_task_api",
                "",
                "pub mod invoice { pub mod api { pub trait Worker {} } }",
            ),
            (
                "shop_task_host",
                "shop_task_api={path=\"../shop_task_api\"}\n[package.metadata.klibs-rules]\nrole=\"core-platform\"\n",
                "mod invoice { mod binding_worker; mod loose_worker; }",
            ),
        ],
        &[
            (
                "shop_task_host/src/invoice/binding_worker.rs",
                "pub(crate) struct BindingWorker; impl shop_task_api::invoice::api::Worker for BindingWorker {}",
            ),
            (
                "shop_task_host/src/invoice/loose_worker.rs",
                "struct LooseWorker;",
            ),
        ],
    );
    let findings = KindShape.check(&context);
    let subjects: Vec<String> = findings
        .iter()
        .map(|finding| finding.finding.subject.clone())
        .collect();
    assert_eq!(
        subjects,
        ["shop_task_host::invoice::loose_worker"],
        "{findings:?}"
    );
}

#[test]
fn given_test_mirror_of_a_file_with_only_a_standard_trait_impl_when_checked_then_the_mirror_is_reported_with_its_source()
 {
    let subjects = list_shape_subjects(
        &format!(
            "mod slot {{ mod guard; mod command_slot; {SLOT_API} }}\n#[cfg(test)]\n#[path = \"../test\"]\nmod test {{ mod slot {{ mod guard_test; mod command_slot_test; }} }}"
        ),
        &[
            (
                "slot/guard.rs",
                "pub(crate) struct Guard; impl Drop for Guard { fn drop(&mut self) {} }",
            ),
            ("slot/command_slot.rs", MIRROR_IMPLEMENTATION),
            (
                "../test/slot/guard_test.rs",
                "#[test] fn given_a_when_b_then_c() {}",
            ),
            (
                "../test/slot/command_slot_test.rs",
                "#[test] fn given_a_when_b_then_c() {}",
            ),
        ],
        Role::CorePlatform,
    );
    assert_eq!(
        subjects,
        [
            "case_sample::slot::guard",
            "case_sample::test::slot::guard_test"
        ]
    );
}

#[test]
fn given_role_kind_fixtures_when_checked_then_exact_wrong_sites_report_and_correct_kinds_pass() {
    let context = read_case_context("KindShape", "fail-role-kind");
    assert_eq!(
        list_findings(&KindShape.check(&context)),
        [
            "modules/sample/src/lib.rs:9 case_sample::vote::parser::ballot_parser",
            "modules/sample/src/vote/limiter/limit_items.rs:3 case_sample::vote::limiter::Window",
            "modules/sample/src/vote/limiter/limit_items.rs:5 case_sample::vote::limiter::WINDOW",
            "modules/sample/src/vote/limiter/limit_items.rs:7 case_sample::vote::limiter::LIMIT",
            "modules/sample/src/vote/limiter/limit_items.rs:9 case_sample::vote::limiter::compute_window",
            "modules/sample/src/vote/limiter/vote_cooldown.rs:1 case_sample::vote::limiter::VoteCooldown",
        ]
    );
    let context = read_case_context("KindShape", "pass-role-kind");
    assert!(KindShape.check(&context).is_empty());
}
