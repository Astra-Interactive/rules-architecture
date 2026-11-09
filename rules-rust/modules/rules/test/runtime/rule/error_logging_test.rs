use crate::runtime::rule::ErrorLogging;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_result_error_inspection_before_propagation_when_checked_then_rejects_log_and_return() {
    assert_code_findings(
        &ErrorLogging,
        "ErrorLogging",
        "fn run(result: Result<(), std::io::Error>) -> Result<(), std::io::Error> {\n result.inspect_err(|error| { tracing::warn!(error = &error as &dyn std::error::Error, \"failed\"); })?;\n Ok(())\n}",
        &[2],
    );
    assert_code_findings(
        &ErrorLogging,
        "ErrorLogging",
        "fn run(result: Result<(), std::io::Error>) {\n let consumed = result.inspect_err(|error| { tracing::warn!(error = &error as &dyn std::error::Error, \"failed\"); }).is_err();\n}",
        &[],
    );
}

#[test]
fn given_formatted_or_repeated_errors_when_checked_then_requires_one_chain_preserving_field() {
    for logger in [
        "log::warn!(\"{}\", error)",
        "tracing::warn!(error = %error, \"failed\")",
        "tracing::warn!(error = ?error, \"failed\")",
    ] {
        assert_code_findings(
            &ErrorLogging,
            "ErrorLogging",
            &format!(
                "fn run(result: Result<(), std::io::Error>) {{\n if let Err(error) = result {{ {logger}; }}\n}}"
            ),
            &[2],
        );
    }
    assert_code_findings(
        &ErrorLogging,
        "ErrorLogging",
        "fn run(result: Result<(), std::io::Error>) {\n if let Err(error) = result {\n tracing::warn!(error = &error as &dyn std::error::Error, \"failed\");\n tracing::error!(error = &error as &dyn std::error::Error, \"failed again\");\n }\n}",
        &[4],
    );
    assert_code_findings(
        &ErrorLogging,
        "ErrorLogging",
        "fn run(result: Result<(), std::io::Error>, ready: bool) {\n if let Err(error) = result {\n if ready { tracing::warn!(error = &error as &dyn std::error::Error, \"failed\"); } else { tracing::error!(error = &error as &dyn std::error::Error, \"failed\"); }\n }\n}",
        &[],
    );
    assert_code_findings(
        &ErrorLogging,
        "ErrorLogging",
        "fn run(result: Result<(), std::io::Error>) { if let Err(error) = result { log::warn!(error:err = error; \"failed\"); } }",
        &[],
    );
}

#[test]
fn given_logged_then_returned_error_and_shadowed_names_when_checked_then_tracks_the_caught_binding()
{
    assert_code_findings(
        &ErrorLogging,
        "ErrorLogging",
        "fn run(result: Result<(), std::io::Error>) -> Result<(), std::io::Error> {\n    if let Err(error) = result {\n        tracing::warn!(error = &error as &dyn std::error::Error, \"Failed\");\n        return Err(error);\n    }\n    Ok(())\n}",
        &[3],
    );
    assert_code_findings(
        &ErrorLogging,
        "ErrorLogging",
        "fn run(result: Result<(), std::io::Error>) {\n    if let Err(error) = result { let error = \"status\"; tracing::warn!(\"Message: {error}\"); }\n}",
        &[],
    );
    assert_code_findings(
        &ErrorLogging,
        "ErrorLogging",
        "enum Event { Err(u32) }\nuse Event::Err;\nfn run() { if let Err(error) = Event::Err(1) { tracing::warn!(\"Value: {error}\"); } }",
        &[],
    );
}

#[test]
fn given_normative_runtime_counterexamples_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &ErrorLogging,
        "ErrorLogging",
        "fn run(result: Result<(), std::io::Error>) {\n    if let Err(error) = result { tracing::warn!(\"Failed: {error}\"); }\n}\n",
        &[2],
    );
    assert_code_findings(
        &ErrorLogging,
        "ErrorLogging",
        "fn run(result: Result<(), std::io::Error>) {\n    if let Err(error) = result { tracing::warn!(error = &error as &dyn std::error::Error, \"Failed\"); }\n}\n",
        &[],
    );
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&ErrorLogging, "ErrorLogging", "bad");
    assert_code_fixture(&ErrorLogging, "ErrorLogging", "good");
}
