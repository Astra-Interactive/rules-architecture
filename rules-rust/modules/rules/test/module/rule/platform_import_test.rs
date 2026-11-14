use crate::check::api::Rule;
use crate::check::model::Anchor;
use crate::config::model::Role;
use crate::module::rule::PlatformImport;
use crate::suppression::model::RuleActivity;
use crate::suppression::util::{apply_suppressions, validate_expectations};
use crate::test::module::util::assert_import_pair;
use crate::test::module::util::assert_import_workspace;
use crate::test::module::util::read_import_context;
use std::collections::BTreeSet;

#[test]
fn given_empty_platform_words_and_two_expectations_when_checked_then_only_the_technology_finding_is_fulfilled()
 {
    let context = read_import_context(
        "mod model {\n #[cfg_attr(klibs_rules, expect(PlatformImport, reason = \"suspected false positive, reported as klibs-rules#301\"))]\n use host_api::Context;\n #[cfg_attr(klibs_rules, expect(PlatformImport, reason = \"suspected false positive, reported as klibs-rules#302\"))]\n use sqlx::Row;\n}",
        Role::Context,
        &[],
        &["sqlx"],
        "app",
    );
    let expectations = context
        .crates
        .iter()
        .flat_map(|sources| sources.expectations.clone())
        .collect::<Vec<_>>();
    let validated = validate_expectations(&expectations);
    let activity = RuleActivity {
        active_rules: BTreeSet::from(["PlatformImport".to_owned()]),
        is_platform_words_empty: true,
    };
    let outcome = apply_suppressions(
        PlatformImport.check(&context),
        validated,
        &context.project_config,
        &activity,
    );
    assert_eq!(outcome.accepted.len(), 1);
    assert_eq!(outcome.violations.len(), 1);
    assert_eq!(outcome.violations[0].rule_id.0, "UnfulfilledExpectation");
    assert!(
        outcome.violations[0]
            .message
            .contains("platform checks are inactive")
    );
}

#[test]
fn given_persistent_phase3_import_pairs_when_checked_then_domains_fail_and_boundaries_pass() {
    assert_import_pair(&PlatformImport, "PlatformImport", Role::Context);
}

#[test]
fn given_real_cargo_aliases_when_checked_then_package_identity_and_exact_anchors_are_preserved() {
    assert_import_workspace(&PlatformImport, "platform-expected.tsv");
}

#[test]
fn given_framework_and_technology_paths_when_checked_then_domains_report_and_boundaries_pass() {
    let context = read_import_context(
        "mod model {\n use host_api::Context;\n use sqlx::Row;\n}\nmod file { use host_api::Context; use sqlx::Row; }\nmod command { use host_api::Context; }",
        Role::Context,
        &["host"],
        &["sqlx"],
        "app",
    );
    let findings = PlatformImport.check(&context);
    assert_eq!(findings.len(), 2);
    assert_eq!(findings[0].finding.location.line, Some(2));
    assert_eq!(findings[1].finding.location.line, Some(3));
    assert!(
        findings
            .iter()
            .all(|finding| matches!(finding.anchor, Anchor::Node { .. }))
    );
}

#[test]
fn given_platform_path_in_test_and_fixture_trees_when_checked_then_only_the_source_tree_reports() {
    let context = read_import_context(
        "mod model {\n use host_api::Context;\n}\n#[cfg(test)]\nmod test {\n mod model {\n use host_api::Context;\n }\n}\n#[cfg(any(test, feature = \"fake\"))]\npub mod fixture {\n pub mod model {\n use host_api::Context;\n }\n}",
        Role::Context,
        &["host"],
        &[],
        "app",
    );
    let findings = PlatformImport.check(&context);
    assert_eq!(1, findings.len(), "{findings:?}");
    assert_eq!(Some(2), findings[0].finding.location.line);
}

#[test]
fn given_empty_platform_words_when_checked_then_technology_paths_still_report_and_local_names_are_distinct()
 {
    let context = read_import_context(
        "mod model { use host_api::Context; use sqlx::Row; } mod sqlx { pub struct Row; } mod util { use crate::sqlx::Row; }",
        Role::Context,
        &[],
        &["sqlx"],
        "app",
    );
    let findings = PlatformImport.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.subject, "sqlx::Row");
}

#[test]
fn given_serde_and_tokio_sync_when_checked_then_only_other_tokio_technology_paths_report() {
    let context = read_import_context(
        "mod model { use serde::Serialize; use tokio::sync::watch; use tokio::runtime::Runtime; }",
        Role::Context,
        &["tokio"],
        &["tokio"],
        "app",
    );
    let findings = PlatformImport.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.subject, "tokio::runtime::Runtime");
}

#[test]
fn given_platform_free_roles_when_checked_then_unit_scope_overrides_mechanism_and_technology_packages()
 {
    for role in [Role::CoreApi, Role::StorageContract, Role::Contract] {
        let context = read_import_context(
            "mod file { use host_api::Context; }",
            role,
            &["host"],
            &[],
            "app",
        );
        assert_eq!(PlatformImport.check(&context).len(), 1, "{role:?}");
    }
    let app = read_import_context(
        "mod command { use host_api::Context; }",
        Role::Api,
        &["host"],
        &[],
        "app",
    );
    assert!(PlatformImport.check(&app).is_empty());
    let plugin = read_import_context(
        "mod command { use host_api::Context; }",
        Role::Api,
        &["host"],
        &[],
        "platform-plugin",
    );
    assert_eq!(PlatformImport.check(&plugin).len(), 1);
}

#[test]
fn given_listener_bridge_types_when_checked_then_framework_registration_signatures_and_event_impls_are_distinguished()
 {
    let context = read_import_context(
        "mod command { trait Setup { fn register(&self, context: host_api::Context); } struct Held { value: framework::Callback } } mod event { struct Listener; impl framework::Listener for Listener {} }",
        Role::Context,
        &["host"],
        &["framework"],
        "app",
    );
    let findings = PlatformImport.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.subject, "framework::Callback");
}

#[test]
fn given_registration_shapes_when_classified_then_base_shape_and_strict_platform_policy_have_distinct_contracts()
 {
    let context = read_import_context(
        "mod route { trait Known { fn register(&self, ctx: host_api::Context); } trait Standard { fn register(&self, ctx: &std::path::Path); } trait Empty { fn register(&self); } } mod api { trait Domain { fn register(&self, ctx: host_api::Context); } }",
        Role::Core,
        &["host"],
        &[],
        "app",
    );
    for item in &context.indexes[0].items {
        assert!(context.has_registration_shape(0, item));
        assert_eq!(
            context.is_platform_registration_trait(0, item),
            item.name == "Known"
        );
    }
}
