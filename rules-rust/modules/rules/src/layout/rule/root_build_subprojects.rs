use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::workspace::model::{Member, TomlValue};

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

    fn check_dependencies(
        member: &Member,
        root: &TomlValue,
        table: &TomlValue,
        prefix: &str,
    ) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for kind in ["dependencies", "dev-dependencies", "build-dependencies"] {
            let Some(dependencies) = table.find(&[kind]) else {
                continue;
            };
            for dependency in dependencies.entries() {
                let is_shared = root
                    .find(&["workspace", "dependencies", &dependency.key])
                    .is_some();
                if is_shared && !Self::inherits(&dependency.value) {
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

    fn check_member(member: &Member, root: &TomlValue) -> Vec<RuleFinding> {
        let manifest = &member.manifest.root;
        let mut findings = Vec::new();
        for key in ["profile", "patch", "replace"] {
            if let Some(entry) = manifest.find_entry(key) {
                findings.push(Self::build_finding(member, key, entry.line));
            }
        }
        for key in [
            "edition",
            "rust-version",
            "license",
            "authors",
            "repository",
        ] {
            let Some(value) = manifest.find(&["package", key]) else {
                continue;
            };
            let is_shared = root.find(&["workspace", "package", key]).is_some();
            if is_shared && !Self::inherits(value) {
                findings.push(Self::build_finding(
                    member,
                    &format!("package.{key}"),
                    value.line,
                ));
            }
        }
        findings.extend(Self::check_dependencies(member, root, manifest, ""));
        if let Some(targets) = manifest.find(&["target"]) {
            for target in targets.entries() {
                findings.extend(Self::check_dependencies(
                    member,
                    root,
                    &target.value,
                    &format!("target.{}.", target.key),
                ));
            }
        }
        findings
    }
}

impl Rule for RootBuildSubprojects {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .workspace
            .members
            .iter()
            .filter(|member| !member.directory.is_empty())
            .flat_map(|member| Self::check_member(member, &context.workspace.root_manifest.root))
            .collect()
    }
}
