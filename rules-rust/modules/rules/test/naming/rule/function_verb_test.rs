use crate::check::api::Rule;
use crate::naming::rule::FunctionVerb;
use crate::test::check::usecase::read_case_context;

#[test]
fn given_trait_implementations_tests_and_lookups_when_checked_then_keeps_exempt_names() {
    crate::test::syntax::util::assert_syntax_findings(
        &FunctionVerb,
        "FunctionVerb",
        "fn get() {} fn get_mut() {} fn get_unchecked() {} fn get_unchecked_mut() {} fn forget_value() {} #[test] fn get_value() {} #[tokio::test] async fn map_value() {} impl Reader for Row { fn process_value() {} } trait Reader { fn handle_value(); } fn map_value() {} fn make_value() {} fn do_work() {} fn process_row() {}",
        5,
    );
}

#[test]
fn given_forbidden_syntax_when_checked_then_reports_the_rule() {
    let context = read_case_context("FunctionVerb", "fail-syntax");
    assert!(!FunctionVerb.check(&context).is_empty());
}

#[test]
fn given_prescribed_syntax_when_checked_then_reports_nothing() {
    let context = read_case_context("FunctionVerb", "pass-syntax");
    assert!(FunctionVerb.check(&context).is_empty());
}
