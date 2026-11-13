use crate::gate::model::GateError;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub(crate) fn collect_document_names(
    docs: &Path,
    entries: impl Iterator<Item = Result<fs::DirEntry, std::io::Error>>,
) -> Result<BTreeSet<String>, GateError> {
    let names = entries
        .map(|entry| {
            entry
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .map_err(|error| GateError::Read {
                    path: docs.to_string_lossy().into_owned(),
                    source: error.into(),
                })
        })
        .collect::<Result<Vec<String>, GateError>>()?;
    Ok(names
        .into_iter()
        .filter(|name| name.ends_with(".rs.md"))
        .collect())
}
