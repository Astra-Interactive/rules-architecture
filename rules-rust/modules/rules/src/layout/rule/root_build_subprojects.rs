use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::workspace::model::{Member, TomlEntry, TomlKind, TomlValue};

pub(crate) struct RootBuildSubprojects;

impl RootBuildSubprojects {
    fn build_finding(member: &Member, key: &str, line: u32) -> RuleFinding {
        RuleFinding {
            finding: Finding {
                rule_id: RuleId("RootBuildSubprojects".to_owned()),
                location: Location {
                    path: member.manifest.path.clone(),
                    line: Some(line),
                },
                subject: format!("{} {key}", member.name),
                message: format!(
                    "declare {key} in the workspace root and inherit shared settings with workspace = true"
                ),
            },
            anchor: Anchor::Crate(member.name.clone()),
        }
    }

    fn inherits(value: &TomlValue) -> bool {
        value.find(&["workspace"]).and_then(TomlValue::as_bool) == Some(true)
    }

    fn is_same_literal(left: &TomlValue, right: &TomlValue) -> bool {
        match &left.kind {
            TomlKind::Array(left_elements) => right.as_array().is_some_and(|right_elements| {
                left_elements.len() == right_elements.len()
                    && left_elements.iter().zip(right_elements).all(
                        |(left_element, right_element)| {
                            Self::is_same_literal(left_element, right_element)
                        },
                    )
            }),
            TomlKind::Table(left_entries) => {
                let right_entries = right.entries();
                right.is_table()
                    && left_entries.len() == right_entries.len()
                    && left_entries
                        .iter()
                        .zip(right_entries)
                        .all(|(left_entry, right_entry)| {
                            left_entry.key == right_entry.key
                                && Self::is_same_literal(&left_entry.value, &right_entry.value)
                        })
            }
            other => *other == right.kind,
        }
    }

    fn find_version(value: &TomlValue) -> Option<&str> {
        value
            .as_str()
            .or_else(|| value.find(&["version"]).and_then(TomlValue::as_str))
    }

    fn find_git_source(value: &TomlValue) -> Option<String> {
        let repository = value.find(&["git"]).and_then(TomlValue::as_str)?;
        let reference = ["rev", "tag", "branch"]
            .iter()
            .find_map(|key| value.find(&[key]).and_then(TomlValue::as_str))
            .unwrap_or_default();
        Some(format!("{repository}#{reference}"))
    }

    fn is_same_requirement(left: &TomlValue, right: &TomlValue) -> bool {
        let is_path = left.find(&["path"]).is_some() || right.find(&["path"]).is_some();
        if is_path {
            return false;
        }
        match (Self::find_git_source(left), Self::find_git_source(right)) {
            (Some(left_source), Some(right_source)) => return left_source == right_source,
            (None, None) => {}
            _ => return false,
        }
        match (Self::find_version(left), Self::find_version(right)) {
            (Some(left_version), Some(right_version)) => left_version == right_version,
            _ => Self::is_same_literal(left, right),
        }
    }

    fn list_dependency_tables(manifest: &TomlValue) -> Vec<&TomlValue> {
        let mut scopes = vec![manifest];
        if let Some(targets) = manifest.find(&["target"]) {
            scopes.extend(targets.entries().iter().map(|target| &target.value));
        }
        scopes
            .into_iter()
            .flat_map(|scope| {
                Self::DEPENDENCY_KINDS
                    .into_iter()
                    .filter_map(move |kind| scope.find(&[kind]))
            })
            .collect()
    }

    fn has_requirement(value: &TomlValue) -> bool {
        value.as_str().is_some()
            || value.find(&["version"]).is_some()
            || value.find(&["git"]).is_some()
    }

    fn is_dependency_repeated(earlier: &[&Member], dependency: &TomlEntry) -> bool {
        earlier.iter().any(|member| {
            Self::list_dependency_tables(&member.manifest.root)
                .into_iter()
                .filter_map(|table| table.find_entry(&dependency.key))
                .any(|other| {
                    !Self::inherits(&other.value)
                        && Self::is_same_requirement(&dependency.value, &other.value)
                })
        })
    }

    fn find_package_name(dependency: &TomlEntry) -> &str {
        dependency
            .value
            .find(&["package"])
            .and_then(TomlValue::as_str)
            .unwrap_or(&dependency.key)
    }

    fn is_declared_in_root(root: &TomlValue, dependency: &TomlEntry) -> bool {
        let Some(declared) = root.find(&["workspace", "dependencies"]) else {
            return false;
        };
        let package_name = Self::find_package_name(dependency);
        declared
            .entries()
            .iter()
            .any(|entry| Self::find_package_name(entry) == package_name)
    }

    fn check_dependencies(
        member: &Member,
        root: &TomlValue,
        table: &TomlValue,
        prefix: &str,
        earlier: &[&Member],
    ) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for kind in Self::DEPENDENCY_KINDS {
            let Some(dependencies) = table.find(&[kind]) else {
                continue;
            };
            for dependency in dependencies.entries() {
                if Self::inherits(&dependency.value) {
                    continue;
                }
                let is_shared = Self::is_declared_in_root(root, dependency);
                let is_repeated = Self::has_requirement(&dependency.value)
                    && Self::is_dependency_repeated(earlier, dependency);
                if is_shared || is_repeated {
                    findings.push(Self::build_finding(
                        member,
                        &format!("{prefix}{kind}.{}", dependency.key),
                        dependency.line,
                    ));
                }
            }
        }
        findings
    }

    fn is_package_key_shared(key: &str, value: &TomlValue, declared: &TomlValue) -> bool {
        Self::INHERITED_PACKAGE_KEYS.contains(&key) || Self::is_same_literal(value, declared)
    }

    fn check_package_keys(member: &Member, root: &TomlValue) -> Vec<RuleFinding> {
        let manifest = &member.manifest.root;
        let Some(package) = root.find(&["workspace", "package"]) else {
            return Vec::new();
        };
        let mut findings = Vec::new();
        for declared in package.entries() {
            let key = declared.key.as_str();
            if Self::PATH_PACKAGE_KEYS.contains(&key) {
                continue;
            }
            let Some(value) = manifest.find(&["package", key]) else {
                continue;
            };
            if !Self::inherits(value) && Self::is_package_key_shared(key, value, &declared.value) {
                findings.push(Self::build_finding(
                    member,
                    &format!("package.{key}"),
                    value.line,
                ));
            }
        }
        findings
    }

    fn check_member(member: &Member, root: &TomlValue, earlier: &[&Member]) -> Vec<RuleFinding> {
        let manifest = &member.manifest.root;
        let mut findings = Vec::new();
        for key in ["profile", "patch", "replace"] {
            if let Some(entry) = manifest.find_entry(key) {
                findings.push(Self::build_finding(member, key, entry.line));
            }
        }
        findings.extend(Self::check_package_keys(member, root));
        findings.extend(Self::check_dependencies(
            member, root, manifest, "", earlier,
        ));
        if let Some(targets) = manifest.find(&["target"]) {
            for target in targets.entries() {
                findings.extend(Self::check_dependencies(
                    member,
                    root,
                    &target.value,
                    &format!("target.{}.", target.key),
                    earlier,
                ));
            }
        }
        findings
    }

    const DEPENDENCY_KINDS: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];
    const INHERITED_PACKAGE_KEYS: [&str; 5] = [
        "edition",
        "rust-version",
        "license",
        "authors",
        "repository",
    ];
    const PATH_PACKAGE_KEYS: [&str; 2] = ["readme", "license-file"];
}

impl Rule for RootBuildSubprojects {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let members: Vec<&Member> = context
            .workspace
            .members
            .iter()
            .filter(|member| !member.directory.is_empty())
            .collect();
        let root = &context.workspace.root_manifest.root;
        members
            .iter()
            .enumerate()
            .flat_map(|(position, member)| Self::check_member(member, root, &members[..position]))
            .collect()
    }
}
