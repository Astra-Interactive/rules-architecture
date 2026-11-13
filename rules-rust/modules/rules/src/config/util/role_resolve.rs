use crate::config::model::Role;
use crate::workspace::model::Member;

const ROLE_NAMES: [&str; 15] = [
    "tooling",
    "proc-macro",
    "entry-point",
    "core-ui",
    "core-platform",
    "core-api",
    "core",
    "contract",
    "storage-contract",
    "api",
    "impl",
    "impl-ui",
    "platform",
    "technology",
    "context",
];
const ROLES: [Role; 15] = [
    Role::Tooling,
    Role::ProcMacro,
    Role::EntryPoint,
    Role::CoreUi,
    Role::CorePlatform,
    Role::CoreApi,
    Role::Core,
    Role::Contract,
    Role::StorageContract,
    Role::Api,
    Role::Impl,
    Role::ImplUi,
    Role::Platform,
    Role::Technology,
    Role::Context,
];

pub(crate) fn parse_role(name: &str) -> Option<Role> {
    ROLE_NAMES
        .iter()
        .position(|role_name| *role_name == name)
        .and_then(|index| ROLES.get(index).copied())
}

pub(crate) fn format_role_name(role: Role) -> &'static str {
    ROLES
        .iter()
        .position(|candidate| *candidate == role)
        .and_then(|index| ROLE_NAMES.get(index).copied())
        .unwrap_or("context")
}

pub(crate) fn list_role_names() -> &'static [&'static str] {
    &ROLE_NAMES
}

pub(crate) fn detect_role(
    member: &Member,
    platform_words: &[String],
    technology_words: &[String],
) -> Option<Role> {
    if member.name == "xtask" {
        return Some(Role::Tooling);
    }
    if member.is_proc_macro() {
        return Some(Role::ProcMacro);
    }
    let is_under_instances = member.directory.starts_with("instances/");
    if is_under_instances || !member.has_rust_library() {
        return Some(Role::EntryPoint);
    }
    let segments: Vec<&str> = member.directory.split('/').collect();
    if segments.starts_with(&["modules", "core", "ui"]) {
        return Some(Role::CoreUi);
    }
    if segments.starts_with(&["modules", "core"]) {
        if segments.len() == 3
            && segments
                .get(2)
                .is_some_and(|word| platform_words.iter().any(|platform| platform == word))
        {
            return Some(Role::CorePlatform);
        }
        return Some(Role::CoreApi);
    }
    let suffix = segments.last().copied().unwrap_or("");
    match suffix {
        "contract" => Some(Role::Contract),
        "dao" => Some(Role::StorageContract),
        "api" => Some(Role::Api),
        "impl" => Some(Role::Impl),
        word if platform_words.iter().any(|platform| platform == word) => Some(Role::Platform),
        word if technology_words.iter().any(|technology| technology == word) => {
            Some(Role::Technology)
        }
        _ => Some(Role::Context),
    }
}
