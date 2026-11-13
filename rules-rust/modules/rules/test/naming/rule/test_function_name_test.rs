use crate::check::api::Rule;
use crate::naming::rule::TestFunctionName;
use crate::test::check::usecase::read_case_context;

#[test]
fn given_test_name_edges_when_checked_then_requires_three_nonempty_parts() {
    crate::test::syntax::util::assert_syntax_findings(
        &TestFunctionName,
        "TestFunctionName",
        "#[test] fn given__when_action_then_result() {} #[test] fn given_state_when__then_result() {} #[test] fn given_state_when_action_then_() {} #[test] fn given_State_when_action_then_result() {} #[tokio::test] async fn run() {} #[test] fn given_state_when_action_then_result() {} fn run() {}",
        5,
    );
}

#[test]
fn given_renamed_tokio_test_attribute_when_checked_then_requires_the_test_name_shape() {
    crate::test::declaration::util::assert_code_findings(
        &TestFunctionName,
        "TestFunctionName",
        "use tokio::test as async_test;\n#[async_test]\nasync fn works() {}\n#[async_test]\nasync fn given_empty_cart_when_order_is_created_then_returns_cart_is_empty() {}\n#[rstest]\nfn works_too() {}\n#[sqlx::test]\nasync fn works_three() {}",
        &[3],
    );
}

#[test]
fn given_forbidden_syntax_when_checked_then_reports_the_rule() {
    let context = read_case_context("TestFunctionName", "fail-syntax");
    assert!(!TestFunctionName.check(&context).is_empty());
}

#[test]
fn given_prescribed_syntax_when_checked_then_reports_nothing() {
    let context = read_case_context("TestFunctionName", "pass-syntax");
    assert!(TestFunctionName.check(&context).is_empty());
}
