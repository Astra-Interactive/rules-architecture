use crate::check::model::CheckError;
use crate::config::model::AllowedDependency;
use crate::config::model::CrateConfig;
use crate::config::model::ManifestException;
use crate::config::model::ManifestScope;
use crate::config::model::ProjectConfig;
use crate::config::model::Role;
use crate::config::util::FIXED_ENTRY_WORDS;
use crate::config::util::FIXED_KIND_WORDS;
use crate::config::util::RESERVED_KIND_WORDS;
use crate::config::util::build_configuration_error;
use crate::config::util::check_known_keys;
use crate::config::util::detect_role;
use crate::config::util::format_role_name;
use crate::config::util::list_role_names;
use crate::config::util::parse_allowed_dependencies;
use crate::config::util::parse_disabled_checks;
use crate::config::util::parse_role;
use crate::config::util::parse_string;
use crate::config::util::parse_string_list;
use crate::workspace::model::DependencyKind;
use crate::workspace::model::Member;
use crate::workspace::model::TomlValue;
use crate::workspace::model::Workspace;
use syn::ext::IdentExt;

const WORKSPACE_KEYS: [&str; 11] = [
    "product-kind",
    "platform-words",
    "prefix",
    "entry-point-words",
    "technology-words",
    "plural-concepts",
    "extension-kinds",
    "role-kinds",
    "lifecycle-traits",
    "root-module",
    "checks",
];
const PACKAGE_KEYS: [&str; 10] = [
    "role",
    "entry-point-words",
    "technology-words",
    "plural-concepts",
    "extension-kinds",
    "role-kinds",
    "composition",
    "implementation-access",
    "host-items",
    "checks",
];
const WORD_LIST_KEYS: [&str; 5] = [
    "entry-point-words",
    "technology-words",
    "plural-concepts",
    "extension-kinds",
    "role-kinds",
];
const WORKSPACE_TABLE: [&str; 3] = ["workspace", "metadata", "klibs-rules"];
const PACKAGE_TABLE: [&str; 3] = ["package", "metadata", "klibs-rules"];
const ROOT_BLOCK_WORDS: [&str; 2] = ["test", "fixture"];
const GENERIC_WORDS: [&str; 16] = [
    "alloc", "async", "common", "core", "derive", "lib", "macros", "rt", "runtime", "rust", "std",
    "sync", "sys", "util", "utils", "",
];

fn list_technology_words(member: &Member, members: &[Member]) -> Vec<String> {
    let mut words = Vec::new();
    for dependency in &member.dependencies {
        let is_member = dependency.find_member(members).is_some();
        if dependency.kind == DependencyKind::Development || is_member {
            continue;
        }
        words.push(dependency.name.replace('-', "_"));
        let first = dependency.name.split(['-', '_']).next().unwrap_or_default();
        if !GENERIC_WORDS.contains(&first) {
            words.push(first.to_owned());
        }
    }
    words
}

fn parse_optional_list(
    table: Option<&TomlValue>,
    key: &str,
    path: &str,
) -> Result<Vec<String>, CheckError> {
    match table.and_then(|table| table.find_entry(key)) {
        Some(entry) => parse_string_list(&entry.value, path, key),
        None => Ok(Vec::new()),
    }
}

fn check_choice(
    table: Option<&TomlValue>,
    key: &str,
    choices: &[&str],
    path: &str,
) -> Result<(), CheckError> {
    let Some(entry) = table.and_then(|table| table.find_entry(key)) else {
        return Ok(());
    };
    let value = parse_string(&entry.value, path, key)?;
    if choices.contains(&value.as_str()) {
        return Ok(());
    }
    Err(build_configuration_error(
        path,
        entry.line,
        format!(
            "`{key}` is \"{value}\": expected one of {}",
            choices.join(", ")
        ),
    ))
}

fn check_word_lists(table: Option<&TomlValue>, path: &str) -> Result<(), CheckError> {
    for key in WORD_LIST_KEYS {
        parse_optional_list(table, key, path)?;
    }
    Ok(())
}

fn check_product_kind(table: &TomlValue, path: &str) -> Result<(), CheckError> {
    if table.find_entry("product-kind").is_none() {
        let message = "`product-kind` is required: \"app\" or \"platform-plugin\"".to_owned();
        return Err(build_configuration_error(path, table.line, message));
    }
    check_choice(
        Some(table),
        "product-kind",
        &["app", "platform-plugin"],
        path,
    )
}

fn describe_unknown_role(name: &str) -> String {
    if name == "tooling" {
        return "the `tooling` role is retired: a binary is an `entry-point` crate".to_owned();
    }
    format!(
        "unknown role \"{name}\"; roles: {}",
        list_role_names().join(", ")
    )
}

fn parse_declared_role(table: Option<&TomlValue>, path: &str) -> Result<Option<Role>, CheckError> {
    let Some(entry) = table.and_then(|table| table.find_entry("role")) else {
        return Ok(None);
    };
    let name = parse_string(&entry.value, path, "role")?;
    parse_role(&name)
        .map(Some)
        .ok_or_else(|| build_configuration_error(path, entry.line, describe_unknown_role(&name)))
}

fn check_role_key(
    table: Option<&TomlValue>,
    key: &str,
    role: Option<Role>,
    expected: Role,
    path: &str,
) -> Result<(), CheckError> {
    let Some(entry) = table.and_then(|table| table.find_entry(key)) else {
        return Ok(());
    };
    if role == Some(expected) {
        return Ok(());
    }
    Err(build_configuration_error(
        path,
        entry.line,
        format!(
            "`{key}` is taken only by a crate whose role is {}",
            format_role_name(expected)
        ),
    ))
}

fn parse_crate_config(
    member: &Member,
    members: &[Member],
    workspace_technology_words: &[String],
    platform_words: &[String],
) -> Result<CrateConfig, CheckError> {
    let path = member.manifest.path.as_str();
    let table = member.manifest.root.find(&PACKAGE_TABLE);
    if let Some(table) = table {
        check_known_keys(table, &PACKAGE_KEYS, path, "[package.metadata.klibs-rules]")?;
    }
    let dependency_words = list_technology_words(member, members);
    let role = parse_declared_role(table, path)?
        .or_else(|| detect_role(member, platform_words, &dependency_words));
    check_role_key(table, "composition", role, Role::ImplUi, path)?;
    check_role_key(table, "implementation-access", role, Role::ImplUi, path)?;
    check_role_key(table, "host-items", role, Role::EntryPoint, path)?;
    check_choice(table, "composition", &["local", "host"], path)?;
    check_choice(table, "implementation-access", &["siblings", "none"], path)?;
    let host_items = parse_optional_list(table, "host-items", path)?;
    let composition = table
        .and_then(|table| table.find(&["composition"]))
        .and_then(TomlValue::as_str)
        .unwrap_or("local")
        .to_owned();
    let implementation_access = table
        .and_then(|table| table.find(&["implementation-access"]))
        .and_then(TomlValue::as_str)
        .unwrap_or("siblings")
        .to_owned();
    check_word_lists(table, path)?;
    let mut technology_words: Vec<String> = ["file", "memory", "system"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    technology_words.extend(dependency_words);
    technology_words.extend_from_slice(workspace_technology_words);
    technology_words.extend(parse_optional_list(table, "technology-words", path)?);
    technology_words
        .retain(|word| !["database", "network", "local", "remote"].contains(&word.as_str()));
    technology_words.sort();
    technology_words.dedup();
    Ok(CrateConfig {
        name: member.name.clone(),
        role,
        host_items,
        technology_words,
        composition,
        implementation_access,
    })
}

fn find_package_checks(member: &Member) -> Option<&TomlValue> {
    member
        .manifest
        .root
        .find(&PACKAGE_TABLE)
        .and_then(|table| table.find(&["checks"]))
}

fn parse_package_exceptions(member: &Member) -> Result<Vec<ManifestException>, CheckError> {
    let Some(checks) = find_package_checks(member) else {
        return Ok(Vec::new());
    };
    let scope = ManifestScope::Package(member.name.clone());
    parse_disabled_checks(checks, &scope, &member.manifest.path)
}

fn parse_package_allowed_dependencies(
    member: &Member,
) -> Result<Vec<AllowedDependency>, CheckError> {
    let Some(checks) = find_package_checks(member) else {
        return Ok(Vec::new());
    };
    parse_allowed_dependencies(checks, &member.name, &member.manifest.path)
}

fn check_workspace_scalars(table: &TomlValue, path: &str) -> Result<(), CheckError> {
    check_product_kind(table, path)?;
    check_word_lists(Some(table), path)?;
    parse_optional_list(Some(table), "lifecycle-traits", path)?;
    for key in ["prefix", "root-module"] {
        if let Some(entry) = table.find_entry(key) {
            parse_string(&entry.value, path, key)?;
        }
    }
    Ok(())
}

fn is_module_name(word: &str) -> bool {
    syn::parse_str::<syn::Ident>(word).is_ok_and(|identifier| identifier.unraw() == word)
}

fn describe_role_kind_problem(
    word: &str,
    entry_words: &[String],
    technology_words: &[String],
) -> Option<&'static str> {
    if !is_module_name(word) {
        return Some("not a valid Rust identifier");
    }
    if ROOT_BLOCK_WORDS.contains(&word) {
        return Some("a block name of the crate root");
    }
    if FIXED_KIND_WORDS.contains(&word) || RESERVED_KIND_WORDS.contains(&word) {
        return Some("a fixed kind");
    }
    if FIXED_ENTRY_WORDS.contains(&word) || entry_words.iter().any(|entry| entry == word) {
        return Some("an entry-point word");
    }
    technology_words
        .iter()
        .any(|technology| technology == word)
        .then_some("a technology word")
}

fn check_role_kind_table(
    table: Option<&TomlValue>,
    path: &str,
    entry_words: &[String],
    technology_words: &[String],
) -> Result<(), CheckError> {
    let Some(entry) = table.and_then(|table| table.find_entry("role-kinds")) else {
        return Ok(());
    };
    for word in parse_string_list(&entry.value, path, "role-kinds")? {
        let Some(collision) = describe_role_kind_problem(&word, entry_words, technology_words)
        else {
            continue;
        };
        return Err(build_configuration_error(
            path,
            entry.line,
            format!(
                "`role-kinds` word \"{word}\" is {collision}; a role kind is a word of its own"
            ),
        ));
    }
    Ok(())
}

fn check_role_kinds(
    workspace_table: &TomlValue,
    workspace_path: &str,
    member: &Member,
    technology_words: &[String],
) -> Result<(), CheckError> {
    let package_table = member.manifest.root.find(&PACKAGE_TABLE);
    let package_path = member.manifest.path.as_str();
    let mut entry_words =
        parse_optional_list(Some(workspace_table), "entry-point-words", workspace_path)?;
    entry_words.extend(parse_optional_list(
        package_table,
        "entry-point-words",
        package_path,
    )?);
    check_role_kind_table(
        Some(workspace_table),
        workspace_path,
        &entry_words,
        technology_words,
    )?;
    check_role_kind_table(package_table, package_path, &entry_words, technology_words)
}

pub(crate) fn parse_project_config(workspace: &Workspace) -> Result<ProjectConfig, CheckError> {
    let path = workspace.root_manifest.path.as_str();
    let Some(table) = workspace.root_manifest.root.find(&WORKSPACE_TABLE) else {
        return Err(CheckError::Configuration {
            path: path.to_owned(),
            line: None,
            message: "[workspace.metadata.klibs-rules] with `product-kind` is required".to_owned(),
        });
    };
    check_known_keys(
        table,
        &WORKSPACE_KEYS,
        path,
        "[workspace.metadata.klibs-rules]",
    )?;
    check_workspace_scalars(table, path)?;
    let mut manifest_exceptions = match table.find_entry("checks") {
        Some(checks) => parse_disabled_checks(&checks.value, &ManifestScope::Workspace, path)?,
        None => Vec::new(),
    };
    let technology_words = parse_optional_list(Some(table), "technology-words", path)?;
    let platform_words = parse_optional_list(Some(table), "platform-words", path)?;
    let mut allowed_dependencies = Vec::new();
    let mut crates = Vec::new();
    for member in &workspace.members {
        let config = parse_crate_config(
            member,
            &workspace.members,
            &technology_words,
            &platform_words,
        )?;
        check_role_kinds(table, path, member, &config.technology_words)?;
        crates.push(config);
        manifest_exceptions.extend(parse_package_exceptions(member)?);
        allowed_dependencies.extend(parse_package_allowed_dependencies(member)?);
    }
    Ok(ProjectConfig {
        platform_words,
        crates,
        manifest_exceptions,
        allowed_dependencies,
    })
}
