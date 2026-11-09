use crate::check::api::Rule;
use crate::naming::rule::ImplementationName;
use crate::test::check::usecase::read_case_context;

#[test]
fn given_type_kinds_when_checked_then_reports_only_forbidden_names() {
    crate::test::syntax::util::assert_syntax_findings(
        &ImplementationName,
        "ImplementationName",
        "struct RowImpl; enum RowInterface {} union RowImplUnion { value: u8 } trait IReader {} trait IO {} trait Invoice {} union ValueImpl { value: u8 }",
        4,
    );
}

#[test]
fn given_forbidden_syntax_when_checked_then_reports_the_rule() {
    let context = read_case_context("ImplementationName", "fail-syntax");
    assert!(!ImplementationName.check(&context).is_empty());
}

#[test]
fn given_prescribed_syntax_when_checked_then_reports_nothing() {
    let context = read_case_context("ImplementationName", "pass-syntax");
    assert!(ImplementationName.check(&context).is_empty());
}
