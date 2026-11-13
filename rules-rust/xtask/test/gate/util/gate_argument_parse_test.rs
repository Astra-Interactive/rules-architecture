use crate::gate::model::GateArguments;
use crate::gate::model::GateError;
use crate::gate::util::parse_gate_arguments;
use std::path::PathBuf;

fn to_arguments(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| (*word).to_owned()).collect()
}

#[test]
fn given_docs_and_check_when_parsed_then_returns_both() {
    let arguments = parse_gate_arguments(&to_arguments(&["--check", "--docs", "docs"]))
        .expect("valid arguments");

    assert_eq!(
        GateArguments {
            docs: PathBuf::from("docs"),
            is_check: true,
        },
        arguments
    );
}

#[test]
fn given_no_docs_when_parsed_then_returns_usage() {
    assert!(matches!(
        parse_gate_arguments(&to_arguments(&["--check"])),
        Err(GateError::Usage)
    ));
}

#[test]
fn given_unknown_argument_when_parsed_then_returns_usage() {
    let result = parse_gate_arguments(&to_arguments(&["--docs", "docs", "--force"]));

    assert!(matches!(result, Err(GateError::Usage)));
}
