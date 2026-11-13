use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::workspace::model::TomlValue;

pub(crate) struct ProjectDirMatchesPath;

impl ProjectDirMatchesPath {
    fn build_finding(manifest_path: &str, line: u32, subject: &str, message: &str) -> RuleFinding {
        RuleFinding {
            finding: Finding {
                rule_id: RuleId("ProjectDirMatchesPath".to_owned()),
                location: Location {
                    path: manifest_path.to_owned(),
                    line: Some(line),
                },
                subject: subject.to_owned(),
                message: message.to_owned(),
            },
            anchor: Anchor::Workspace,
        }
    }

    fn is_listed(directory: &str, listed: &[&str]) -> bool {
        listed.iter().any(|path| {
            let path = path.trim_start_matches("./").trim_end_matches('/');
            match path.find(['*', '?', '[']) {
                Some(position) => directory.starts_with(&path[..position]),
                None => path == directory,
            }
        })
    }
}

impl Rule for ProjectDirMatchesPath {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let manifest = &context.workspace.root_manifest;
        let Some(members) = manifest
            .root
            .find(&["workspace"])
            .and_then(|workspace| workspace.find_entry("members"))
        else {
            return Vec::new();
        };
        let entries = members.value.as_array().unwrap_or_default();
        let listed: Vec<&str> = entries.iter().filter_map(TomlValue::as_str).collect();
        let mut findings: Vec<RuleFinding> = entries
            .iter()
            .filter_map(|entry| {
                let path = entry.as_str()?;
                path.contains(['*', '?', '[']).then(|| {
                    Self::build_finding(
                        &manifest.path,
                        entry.line,
                        path,
                        "list each member path explicitly: workspace members use no glob",
                    )
                })
            })
            .collect();
        findings.extend(
            context
                .workspace
                .members
                .iter()
                .filter(|member| {
                    !member.directory.is_empty() && !Self::is_listed(&member.directory, &listed)
                })
                .map(|member| {
                    Self::build_finding(
                        &manifest.path,
                        members.line,
                        &member.directory,
                        "list each member path explicitly: a crate reached only by a path dependency is still a workspace member",
                    )
                }),
        );
        findings
    }
}
