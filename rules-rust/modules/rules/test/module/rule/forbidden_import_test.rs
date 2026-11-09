use crate::check::api::Rule;
use crate::check::model::Anchor;
use crate::config::model::Role;
use crate::module::rule::ForbiddenImport;
use crate::test::module::util::assert_import_pair;
use crate::test::module::util::assert_import_workspace;
use crate::test::module::util::read_import_context;

#[test]
fn given_persistent_phase3_import_pairs_when_checked_then_domains_fail_and_boundaries_pass() {
    assert_import_pair(&ForbiddenImport, "ForbiddenImport", Role::Context);
}

#[test]
fn given_renamed_real_time_dependency_when_checked_then_calendar_alias_cannot_hide_chrono() {
    assert_import_workspace(&ForbiddenImport, "time-expected.tsv");
}

#[test]
fn given_time_crate_paths_when_checked_then_domain_paths_report_and_technology_boundaries_pass() {
    let context = read_import_context(
        "mod model {\n use chrono::DateTime;\n use time::Duration;\n use jiff::Timestamp;\n}\nmod file { use chrono::DateTime; use time::Duration; use jiff::Timestamp; }",
        Role::Context,
        &[],
        &[],
        "app",
    );
    let findings = ForbiddenImport.check(&context);
    assert_eq!(findings.len(), 3);
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.finding.location.line)
            .collect::<Vec<_>>(),
        [Some(2), Some(3), Some(4)]
    );
    assert!(
        findings
            .iter()
            .all(|finding| matches!(finding.anchor, Anchor::Node { .. }))
    );
}

#[test]
fn given_time_crate_boundaries_and_local_names_when_checked_then_only_external_identity_is_restricted()
 {
    for role in [Role::Technology, Role::Contract] {
        let context =
            read_import_context("mod model { use chrono::DateTime; }", role, &[], &[], "app");
        assert!(ForbiddenImport.check(&context).is_empty());
    }
    let context = read_import_context(
        "mod chrono { pub struct DateTime; } mod model { use crate::chrono::DateTime; }",
        Role::Context,
        &[],
        &[],
        "app",
    );
    assert!(ForbiddenImport.check(&context).is_empty());
}
