use crate::runtime::rule::ErrorLogging;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_error_taken_by_err_or_unwrap_err_when_formatted_into_a_message_then_reports_the_log() {
    assert_code_findings(
        &ErrorLogging,
        "ErrorLogging",
        "fn run(outcome: Result<(), std::io::Error>, queue: &mut Vec<String>) {\n    if let Some(error) = outcome.err() {\n        tracing::warn!(\"could not sync: {error}\");\n    }\n    if let Some(item) = queue.pop() {\n        tracing::warn!(\"dropped {item}\");\n    }\n}\nfn load(outcome: Result<(), std::io::Error>) {\n    let error = outcome.unwrap_err();\n    tracing::warn!(\"could not load: {error}\");\n}",
        &[3, 11],
    );
}

#[test]
fn given_logged_error_returned_inside_a_wrapper_when_checked_then_reports_each_log() {
    assert_code_findings(
        &ErrorLogging,
        "ErrorLogging",
        "fn load() -> Result<(), std::io::Error> { Ok(()) }\nenum SyncError { Io { source: std::io::Error }, Parse }\nimpl From<std::io::Error> for SyncError { fn from(source: std::io::Error) -> Self { Self::Io { source } } }\nfn sync() -> Result<(), SyncError> {\n    if let Err(error) = load() {\n        tracing::warn!(error = &error as &dyn std::error::Error, \"could not load\");\n        return Err(SyncError::Io { source: error });\n    }\n    if let Err(error) = load() {\n        tracing::warn!(error = &error as &dyn std::error::Error, \"could not load\");\n        return Err(error.into());\n    }\n    if let Err(error) = load() {\n        tracing::warn!(error = &error as &dyn std::error::Error, \"could not load\");\n        return Err(SyncError::from(error));\n    }\n    if let Err(error) = load() {\n        tracing::warn!(error = &error as &dyn std::error::Error, \"could not load\");\n        return Err(SyncError::Parse);\n    }\n    Ok(())\n}",
        &[6, 10, 14],
    );
}

#[test]
fn given_error_logged_in_if_let_then_its_result_propagated_with_try_when_checked_then_reports_the_log()
 {
    assert_code_findings(
        &ErrorLogging,
        "ErrorLogging",
        "fn load() -> Result<u32, std::io::Error> { Ok(1) }\nfn sync() -> Result<u32, std::io::Error> {\n    let result = load();\n    if let Err(error) = &result {\n        tracing::warn!(error = error as &dyn std::error::Error, \"could not load\");\n    }\n    let value = result?;\n    Ok(value)\n}\nfn sync_quiet() -> Result<u32, std::io::Error> {\n    let result = load();\n    if let Err(error) = &result {\n        tracing::warn!(error = error as &dyn std::error::Error, \"could not load\");\n    }\n    let value = result.unwrap_or_default();\n    Ok(value)\n}",
        &[5],
    );
}

#[test]
fn given_shadowed_result_in_pattern_scopes_when_inspected_then_does_not_infer_custom_errors() {
    let declarations = "fn load() -> Result<(), std::io::Error> { Ok(()) } struct Value; impl Value { fn map_err(&self, callback: impl Fn(u32)) { callback(1); } }";
    for body in [
        "let inspect = |value: Value| value.map_err(|number| tracing::info!(\"{number}\"));",
        "match Value { value => value.map_err(|number| tracing::info!(\"{number}\")) }",
        "for value in [Value] { value.map_err(|number| tracing::info!(\"{number}\")); }",
        "if let Some(value) = Some(Value) { value.map_err(|number| tracing::info!(\"{number}\")); }",
        "while let Some(value) = Some(Value) { value.map_err(|number| tracing::info!(\"{number}\")); break; }",
    ] {
        assert_code_findings(
            &ErrorLogging,
            "ErrorLogging",
            &format!("{declarations}\nfn run() {{ let value = load(); {body} }}"),
            &[],
        );
    }
}

#[test]
fn given_local_and_direct_result_inspection_when_propagated_then_checks_logging() {
    for receiver in ["result", "load()", "std::fs::read_to_string(\"file\")"] {
        assert_code_findings(
            &ErrorLogging,
            "ErrorLogging",
            &format!(
                "fn load() -> Result<String, std::io::Error> {{ Ok(String::new()) }}\nfn run() -> Result<(), std::io::Error> {{\n let result = load();\n {receiver}.inspect_err(|error| {{ tracing::warn!(error = (&error as &dyn std::error::Error), \"failed\"); }})?;\n Ok(())\n}}"
            ),
            &[4],
        );
    }
    assert_code_findings(
        &ErrorLogging,
        "ErrorLogging",
        "fn run(result: Result<(), std::io::Error>) {\n { let result = 1; }\n let result = result;\n result.inspect_err(|error| { tracing::warn!(error = ((&error as &dyn std::error::Error)), \"failed\"); }).is_err();\n}",
        &[],
    );
    assert_code_findings(
        &ErrorLogging,
        "ErrorLogging",
        "struct Custom; impl Custom { fn inspect_err(self, action: impl Fn(&str)) {} }\nfn run(result: Result<(), std::io::Error>) {\n let result = Custom;\n result.inspect_err(|error| { tracing::warn!(\"{error}\"); });\n}",
        &[],
    );
}

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

#[test]
fn given_error_bound_by_fallback_closure_when_logged_in_message_then_reports_log() {
    for call in [
        "unwrap_or_else(|error| { tracing::warn!(\"Could not parse: {error}\"); 0 })",
        "or_else(|error| { tracing::warn!(\"Could not parse: {error}\"); Ok::<u32, std::num::ParseIntError>(0) }).unwrap_or(0)",
        "map_or_else(|error| { tracing::warn!(\"Could not parse: {error}\"); 0 }, |value| value)",
    ] {
        assert_code_findings(
            &ErrorLogging,
            "ErrorLogging",
            &format!("fn total(raw: &str) -> u32 {{\n    raw.parse::<u32>().{call}\n}}"),
            &[2],
        );
    }
    assert_code_findings(
        &ErrorLogging,
        "ErrorLogging",
        "fn total(raw: &str) -> u32 {\n    raw.parse::<u32>().unwrap_or_else(|error| { tracing::warn!(error = &error as &dyn std::error::Error, \"Could not parse\"); 0 })\n}",
        &[],
    );
}

#[test]
fn given_error_bound_by_let_else_when_logged_in_message_then_reports_log() {
    assert_code_findings(
        &ErrorLogging,
        "ErrorLogging",
        "fn run(result: Result<(), std::io::Error>) {\n    let Err(error) = result else { return; };\n    tracing::warn!(\"Failed: {error}\");\n}",
        &[3],
    );
}

#[test]
fn given_error_bound_by_while_let_when_logged_in_message_then_reports_log() {
    assert_code_findings(
        &ErrorLogging,
        "ErrorLogging",
        "fn run(result: Result<(), std::io::Error>) {\n    while let Err(error) = result {\n        tracing::warn!(\"Failed: {error}\");\n        break;\n    }\n}",
        &[3],
    );
}

#[test]
fn given_error_bound_in_let_chain_when_logged_in_message_then_reports_log() {
    assert_code_findings(
        &ErrorLogging,
        "ErrorLogging",
        "fn run(result: Result<(), std::io::Error>, attempts: u32) {\n    if let Err(error) = result && attempts > 0 {\n        tracing::warn!(\"Failed: {error}\");\n    }\n}",
        &[3],
    );
}

#[test]
fn given_error_bound_in_nested_pattern_when_logged_in_message_then_reports_log() {
    assert_code_findings(
        &ErrorLogging,
        "ErrorLogging",
        "fn run(result: Option<Result<(), std::io::Error>>) {\n    match result {\n        Some(Err(error)) => tracing::warn!(\"Failed: {error}\"),\n        _ => {}\n    }\n}",
        &[3],
    );
}

#[test]
fn given_result_parameter_of_impl_method_when_error_is_logged_in_message_then_reports_log() {
    assert_code_findings(
        &ErrorLogging,
        "ErrorLogging",
        "struct Loader;\nimpl Loader {\n    fn settle(&self, outcome: Result<u32, std::io::Error>) -> Option<u32> {\n        outcome.map_err(|error| tracing::warn!(\"failed: {error}\")).ok()\n    }\n}\nfn settle(outcome: Result<u32, std::io::Error>) -> Option<u32> {\n    outcome.map_err(|error| tracing::warn!(\"failed: {error}\")).ok()\n}",
        &[4, 8],
    );
}

#[test]
fn given_tdd_error_logging_parsed_error_before_propagation_when_checked_then_reports_log() {
    assert_code_findings(
        &ErrorLogging,
        "ErrorLogging",
        "fn parse(raw: &str) -> Result<u32, std::num::ParseIntError> {\n    let value = raw.parse::<u32>().map_err(|error| { tracing::warn!(error = &error as &dyn std::error::Error, \"parse failed\"); error })?;\n    Ok(value)\n}",
        &[2],
    );
}

#[test]
fn given_tdd_error_logging_parsed_error_with_fallback_when_matched_then_requires_structured_field()
{
    assert_code_findings(
        &ErrorLogging,
        "ErrorLogging",
        "fn parse_or_zero(raw: &str) -> u32 {\n    match raw.parse::<u32>() { Ok(value) => value, Err(error) => { tracing::warn!(\"{error}\"); 0 } }\n}",
        &[2],
    );
}

#[test]
fn given_map_err_on_standard_library_call_when_error_is_logged_in_message_then_reports_log() {
    assert_code_findings(
        &ErrorLogging,
        "ErrorLogging",
        "pub(crate) fn open_config(path: &std::path::Path) -> Option<std::fs::File> {\n    std::fs::File::open(path)\n        .map_err(|error| tracing::warn!(\"Could not open config: {error}\"))\n        .ok()\n}\npub(crate) fn open_checked(path: &std::path::Path) -> Option<std::fs::File> {\n    std::fs::File::open(path)\n        .map_err(|error| tracing::warn!(error = &error as &dyn std::error::Error, \"Could not open config\"))\n        .ok()\n}",
        &[3],
    );
}
