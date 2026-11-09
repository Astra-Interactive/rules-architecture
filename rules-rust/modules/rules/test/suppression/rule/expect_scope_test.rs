use crate::check::api::Rule;
use crate::suppression::rule::ExpectScope;
use crate::test::check::usecase::read_case_context;

#[test]
fn given_unrelated_or_empty_attributes_when_checked_then_reports_no_lint_expectation() {
    crate::test::syntax::util::assert_syntax_findings(
        &ExpectScope,
        "ExpectScope",
        "#[other] #[other(=>)] #[other(value)] #[expect(reason = \"fixture\")] fn run() {}",
        0,
    );
}

#[test]
fn given_inner_module_or_nonprescribed_expectations_when_checked_then_reports_scope() {
    crate::test::syntax::util::assert_syntax_findings(
        &ExpectScope,
        "ExpectScope",
        "#![allow(dead_code)] #![expect(clippy::unwrap_used, reason = \"fixture\")] #[expect(clippy::unwrap_used, reason = \"fixture\")] mod nested { #[expect(clippy::expect_used, reason = \"fixture\")] fn valid() {} } #[cfg_attr(feature = \"case\", expect(dead_code))] fn invalid() {}",
        4,
    );
}

#[test]
fn given_prescribed_lints_and_architecture_expectations_when_checked_then_keeps_them() {
    crate::test::syntax::util::assert_syntax_findings(
        &ExpectScope,
        "ExpectScope",
        "#[cfg_attr(klibs_rules, expect(UnknownRule, reason = \"fixture\"))] mod nested {} #[expect(clippy::unwrap_used, clippy::expect_used, clippy::let_underscore_must_use, clippy::disallowed_methods, clippy::disallowed_types, clippy::print_stdout, reason = \"fixture\")] fn valid() {} #[cfg_attr(feature = \"case\", expect(clippy::expect_used, reason = \"fixture\"))] fn conditional() {}",
        0,
    );
}

#[test]
fn given_forbidden_syntax_when_checked_then_reports_the_rule() {
    let context = read_case_context("ExpectScope", "fail-syntax");
    assert!(!ExpectScope.check(&context).is_empty());
}

#[test]
fn given_prescribed_syntax_when_checked_then_reports_nothing() {
    let context = read_case_context("ExpectScope", "pass-syntax");
    assert!(ExpectScope.check(&context).is_empty());
}
