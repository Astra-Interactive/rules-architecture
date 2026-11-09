use crate::check::api::Rule;
use crate::naming::rule::ClosureParameterName;
use crate::test::check::usecase::read_case_context;

#[test]
fn given_destructured_closure_bindings_when_checked_then_reports_only_single_letters() {
    crate::test::syntax::util::assert_syntax_findings(
        &ClosureParameterName,
        "ClosureParameterName",
        "fn run(x: usize) { let _ = |(i, value), Row { x, long_name }, _unused, _| { let z = i; z }; }",
        2,
    );
}

#[test]
fn given_forbidden_syntax_when_checked_then_reports_the_rule() {
    let context = read_case_context("ClosureParameterName", "fail-syntax");
    assert!(!ClosureParameterName.check(&context).is_empty());
}

#[test]
fn given_prescribed_syntax_when_checked_then_reports_nothing() {
    let context = read_case_context("ClosureParameterName", "pass-syntax");
    assert!(ClosureParameterName.check(&context).is_empty());
}
