use crate::gate::model::GateArguments;
use crate::gate::model::GateError;
use std::path::PathBuf;

pub(crate) fn parse_gate_arguments(arguments: &[String]) -> Result<GateArguments, GateError> {
    let docs = arguments
        .iter()
        .position(|argument| argument == "--docs")
        .and_then(|index| arguments.get(index + 1))
        .ok_or(GateError::Usage)?;
    let is_known =
        |argument: &String| argument == "--docs" || argument == "--check" || argument == docs;
    if !arguments.iter().all(is_known) {
        return Err(GateError::Usage);
    }
    Ok(GateArguments {
        docs: PathBuf::from(docs),
        is_check: arguments.iter().any(|argument| argument == "--check"),
    })
}
