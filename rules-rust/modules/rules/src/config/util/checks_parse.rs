use crate::catalogue::util::find_catalogue_entry;
use crate::catalogue::util::find_retired_rule;
use crate::check::model::CheckError;
use crate::config::model::AllowedDependency;
use crate::config::model::ManifestException;
use crate::config::model::ManifestScope;
use crate::config::util::build_configuration_error;
use crate::config::util::check_known_keys;
use crate::config::util::parse_reason;
use crate::workspace::model::TomlValue;

const WORKSPACE_CHECK_KEYS: [&str; 1] = ["disabled"];
const PACKAGE_CHECK_KEYS: [&str; 2] = ["disabled", "allowed-dependencies"];

fn check_rule_id(
    rule_id: &str,
    scope: &ManifestScope,
    path: &str,
    line: u32,
) -> Result<(), CheckError> {
    if let Some(retired) = find_retired_rule(rule_id) {
        let message = format!("`{rule_id}` is retired: use `{}`", retired.replacement);
        return Err(build_configuration_error(path, line, message));
    }
    let Some(entry) = find_catalogue_entry(rule_id) else {
        return Err(build_configuration_error(
            path,
            line,
            format!("unknown rule id `{rule_id}`"),
        ));
    };
    let (holds, table) = match scope {
        ManifestScope::Package(_) => (entry.anchors.krate, "[package.metadata.klibs-rules.checks]"),
        ManifestScope::Workspace => (
            entry.anchors.workspace,
            "[workspace.metadata.klibs-rules.checks]",
        ),
    };
    if holds {
        return Ok(());
    }
    Err(build_configuration_error(
        path,
        line,
        format!("`{rule_id}` has no finding a {table} table can hold"),
    ))
}

fn check_checks_keys(
    checks: &TomlValue,
    scope: &ManifestScope,
    path: &str,
) -> Result<(), CheckError> {
    match scope {
        ManifestScope::Package(_) => check_known_keys(checks, &PACKAGE_CHECK_KEYS, path, "checks"),
        ManifestScope::Workspace => check_known_keys(checks, &WORKSPACE_CHECK_KEYS, path, "checks"),
    }
}

pub(crate) fn parse_disabled_checks(
    checks: &TomlValue,
    scope: &ManifestScope,
    path: &str,
) -> Result<Vec<ManifestException>, CheckError> {
    check_checks_keys(checks, scope, path)?;
    let Some(disabled) = checks.find_entry("disabled") else {
        return Ok(Vec::new());
    };
    let mut exceptions = Vec::new();
    for rule in disabled.value.entries() {
        check_rule_id(&rule.key, scope, path, rule.line)?;
        if !rule.value.is_table() {
            let message = format!(
                "`disabled.{}` must be a table of \"<subject>\" = \"<reason>\"",
                rule.key
            );
            return Err(build_configuration_error(path, rule.line, message));
        }
        for subject in rule.value.entries() {
            exceptions.push(ManifestException {
                scope: scope.clone(),
                rule_id: rule.key.clone(),
                subject: subject.key.clone(),
                reason: parse_reason(&subject.value, path, &subject.key)?,
                path: path.to_owned(),
                line: subject.line,
            });
        }
    }
    Ok(exceptions)
}

pub(crate) fn parse_allowed_dependencies(
    checks: &TomlValue,
    crate_name: &str,
    path: &str,
) -> Result<Vec<AllowedDependency>, CheckError> {
    let Some(allowed) = checks.find_entry("allowed-dependencies") else {
        return Ok(Vec::new());
    };
    if !allowed.value.is_table() {
        let message =
            "`allowed-dependencies` must be a table of \"<crate>\" = \"<reason>\"".to_owned();
        return Err(build_configuration_error(path, allowed.line, message));
    }
    allowed
        .value
        .entries()
        .iter()
        .map(|dependency| {
            Ok(AllowedDependency {
                from_crate: crate_name.to_owned(),
                to_crate: dependency.key.clone(),
                reason: parse_reason(&dependency.value, path, &dependency.key)?,
                path: path.to_owned(),
                line: dependency.line,
            })
        })
        .collect()
}
