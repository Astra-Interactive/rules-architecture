use crate::check::api::Rule;
use crate::module::rule::ApiConfiguration;
use crate::test::check::usecase::read_case_context;
use crate::test::module::util::assert_resolution_cases;
use crate::test::source::syn::build_index;

#[test]
fn given_persistent_resolution_cases_when_checked_then_external_reexport_is_the_only_rule_finding()
{
    assert_resolution_cases(
        &ApiConfiguration,
        "ApiConfiguration",
        "modules/sample/src/lib.rs:2\tstd::time::Duration\n",
    );
}

#[test]
fn given_external_and_internal_exports_when_checked_then_only_external_reexports_are_reported() {
    let mut context = read_case_context("ImportOrdering", "pass-syntax");
    context.indexes = vec![build_index(
        "mod own { pub struct Thing; } pub(crate) use own::Thing; pub use renamed_dependency::Thing as External; use std::fmt::Debug;",
    )];
    let findings = ApiConfiguration.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.subject, "renamed_dependency::Thing");
}
