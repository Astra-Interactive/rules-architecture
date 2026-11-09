use crate::main;
use std::process::ExitCode;

#[test]
fn given_test_invocation_without_a_task_when_main_runs_then_usage_is_returned() {
    assert_eq!(main(), ExitCode::from(2));
}
