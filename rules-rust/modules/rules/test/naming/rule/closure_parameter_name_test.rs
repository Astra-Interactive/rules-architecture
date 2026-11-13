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
fn given_single_letter_closure_parameters_inside_crossbeam_select_when_checked_then_reports_them() {
    crate::test::declaration::util::assert_code_findings(
        &ClosureParameterName,
        "ClosureParameterName",
        "pub(crate) fn drain(first: crossbeam_channel::Receiver<Vec<u32>>, out: crossbeam_channel::Sender<u32>) {\n    crossbeam_channel::select! {\n        recv(first) -> message => {\n            let doubled: Vec<u32> = message.iter().map(|v| v * 2).collect();\n            let named: Vec<u32> = message.iter().map(|value| value + 1).collect();\n        }\n        send(out, 1) -> sent => drop(sent.iter().map(|n| n)),\n        default => {}\n    }\n}",
        &[4, 7],
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
