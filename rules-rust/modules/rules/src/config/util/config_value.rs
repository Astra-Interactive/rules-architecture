use crate::check::model::CheckError;
use crate::workspace::model::TomlValue;

pub(crate) fn build_configuration_error(path: &str, line: u32, message: String) -> CheckError {
    CheckError::Configuration {
        path: path.to_owned(),
        line: Some(line),
        message,
    }
}

pub(crate) fn parse_string(value: &TomlValue, path: &str, key: &str) -> Result<String, CheckError> {
    value.as_str().map(ToOwned::to_owned).ok_or_else(|| {
        build_configuration_error(path, value.line, format!("`{key}` must be a string"))
    })
}

pub(crate) fn parse_string_list(
    value: &TomlValue,
    path: &str,
    key: &str,
) -> Result<Vec<String>, CheckError> {
    let error = || {
        build_configuration_error(
            path,
            value.line,
            format!("`{key}` must be an array of strings"),
        )
    };
    value
        .as_array()
        .ok_or_else(error)?
        .iter()
        .map(|element| element.as_str().map(ToOwned::to_owned).ok_or_else(error))
        .collect()
}

pub(crate) fn parse_reason(
    value: &TomlValue,
    path: &str,
    subject: &str,
) -> Result<String, CheckError> {
    let reason = value.as_str().ok_or_else(|| {
        build_configuration_error(
            path,
            value.line,
            format!("the reason for \"{subject}\" must be a string"),
        )
    })?;
    if reason.trim().is_empty() {
        return Err(build_configuration_error(
            path,
            value.line,
            format!("the reason for \"{subject}\" is blank"),
        ));
    }
    Ok(reason.to_owned())
}

pub(crate) fn check_known_keys(
    table: &TomlValue,
    allowed: &[&str],
    path: &str,
    table_name: &str,
) -> Result<(), CheckError> {
    if !table.is_table() {
        return Err(build_configuration_error(
            path,
            table.line,
            format!("`{table_name}` must be a table"),
        ));
    }
    let Some(unknown) = table
        .entries()
        .iter()
        .find(|entry| !allowed.contains(&entry.key.as_str()))
    else {
        return Ok(());
    };
    Err(build_configuration_error(
        path,
        unknown.line,
        format!(
            "unknown key `{}` in {table_name}; allowed: {}",
            unknown.key,
            allowed.join(", ")
        ),
    ))
}
