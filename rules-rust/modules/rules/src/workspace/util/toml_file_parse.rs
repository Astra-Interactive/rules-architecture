use crate::check::model::CheckError;
use crate::workspace::model::TomlEntry;
use crate::workspace::model::TomlFile;
use crate::workspace::model::TomlKind;
use crate::workspace::model::TomlValue;
use toml::Spanned;
use toml::de::DeTable;
use toml::de::DeValue;

fn find_line_of_offset(text: &str, offset: usize) -> u32 {
    let preceding_newlines = text
        .as_bytes()
        .iter()
        .take(offset)
        .filter(|byte| **byte == b'\n')
        .count();
    u32::try_from(preceding_newlines).map_or(u32::MAX, |newlines| newlines.saturating_add(1))
}

fn convert_value(text: &str, value: &Spanned<DeValue<'_>>) -> TomlValue {
    let line = find_line_of_offset(text, value.span().start);
    let kind = match value.get_ref() {
        DeValue::String(string) => TomlKind::String(string.to_string()),
        DeValue::Integer(_) => TomlKind::Integer(text[value.span()].to_owned()),
        DeValue::Float(_) => TomlKind::Float(text[value.span()].to_owned()),
        DeValue::Boolean(boolean) => TomlKind::Boolean(*boolean),
        DeValue::Datetime(datetime) => TomlKind::Datetime(datetime.to_string()),
        DeValue::Array(array) => TomlKind::Array(
            array
                .iter()
                .map(|element| convert_value(text, element))
                .collect(),
        ),
        DeValue::Table(table) => convert_table(text, table),
    };
    TomlValue { line, kind }
}

fn convert_table(text: &str, table: &DeTable<'_>) -> TomlKind {
    let entries = table
        .iter()
        .map(|(key, value)| TomlEntry {
            key: key.get_ref().to_string(),
            line: find_line_of_offset(text, key.span().start),
            value: convert_value(text, value),
        })
        .collect();
    TomlKind::Table(entries)
}

pub(crate) fn parse_toml_file(path: &str, text: &str) -> Result<TomlFile, CheckError> {
    let table = DeTable::parse(text).map_err(|error| CheckError::Configuration {
        path: path.to_owned(),
        line: error
            .span()
            .map(|span| find_line_of_offset(text, span.start)),
        message: error.message().to_owned(),
    })?;
    Ok(TomlFile {
        path: path.to_owned(),
        root: TomlValue {
            line: 1,
            kind: convert_table(text, table.get_ref()),
        },
    })
}
