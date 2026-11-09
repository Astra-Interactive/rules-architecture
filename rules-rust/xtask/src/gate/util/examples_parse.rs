use crate::gate::model::ExampleEntry;
use crate::gate::model::ExampleFragment;
use crate::gate::model::ExamplesFile;
use crate::gate::model::GateError;
use crate::gate::model::Translation;
use toml::Table;
use toml::Value;

fn read_string(table: &Value, key: &str) -> Result<String, GateError> {
    table
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| GateError::Examples {
            message: format!("`{key}` must be a string"),
        })
}

fn parse_translation(value: &Value) -> Result<Translation, GateError> {
    Ok(Translation {
        from: read_string(value, "from")?,
        to: read_string(value, "to")?,
    })
}

fn parse_fragment(value: &Value) -> Result<ExampleFragment, GateError> {
    let range = read_string(value, "block-lines")?;
    let invalid = || GateError::Examples {
        message: format!("`block-lines = \"{range}\"` is not `<first>-<last>`"),
    };
    let (first, last) = range.split_once('-').ok_or_else(invalid)?;
    let translations = value
        .get("translate")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .map(parse_translation)
        .collect::<Result<Vec<Translation>, GateError>>()?;
    Ok(ExampleFragment {
        first_block_line: first.parse().map_err(|_| invalid())?,
        last_block_line: last.parse().map_err(|_| invalid())?,
        suffix: value
            .get("suffix")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        translations,
    })
}

fn parse_entry(value: &Value) -> Result<ExampleEntry, GateError> {
    let fragments = value
        .get("fragment")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .map(parse_fragment)
        .collect::<Result<Vec<ExampleFragment>, GateError>>()?;
    Ok(ExampleEntry {
        document: read_string(value, "document")?,
        anchor: read_string(value, "anchor")?,
        first_line: read_string(value, "first-line")?,
        occurrence: value
            .get("occurrence")
            .and_then(Value::as_integer)
            .and_then(|occurrence| usize::try_from(occurrence).ok())
            .unwrap_or(1),
        wrap: read_string(value, "wrap")?,
        sha256: read_string(value, "sha256")?,
        fragments,
    })
}

pub(crate) fn parse_examples(text: &str) -> Result<ExamplesFile, GateError> {
    let table: Table = text
        .parse()
        .map_err(|error: toml::de::Error| GateError::Examples {
            message: error.to_string(),
        })?;
    let commit = table
        .get("sync")
        .and_then(|sync| sync.get("conf-ide-commit"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let entries = table
        .get("example")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .map(parse_entry)
        .collect::<Result<Vec<ExampleEntry>, GateError>>()?;
    Ok(ExamplesFile { commit, entries })
}
