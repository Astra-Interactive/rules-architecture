use crate::check::api::Rule;
use crate::module::rule::KindShape;
use crate::test::check::usecase::read_case_context;
use crate::test::module::util::read_shape_context;

#[test]
fn given_technology_concept_implementations_and_private_helpers_when_checked_then_only_unowned_helper_reports()
 {
    let mut context = read_shape_context(
        "mod invoice { mod repository; mod refreshing_cache; mod unrelated; mod model { struct Invoice; } }",
        &[
            (
                "invoice/repository.rs",
                "struct InvoiceStore { cache: super::refreshing_cache::RefreshingCache } impl std::fmt::Display for InvoiceStore {}",
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
fn given_cfg_specific_field_alias_when_checked_then_incompatible_watch_sender_alias_is_not_used() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::KindShape,
        "mod model { #[cfg(unix)] type Updates = tokio::sync::watch::Sender<u64>; #[cfg(not(unix))] type Updates = String; struct State { #[cfg(not(unix))] updates: Updates } }",
        &[],
    );
}
