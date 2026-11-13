use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::config::model::Role;
use crate::workspace::model::{Member, TomlValue};
use std::collections::BTreeMap;

pub(crate) struct RootPackagePath;

impl RootPackagePath {
    fn is_tooling(context: &RuleContext, member: &Member) -> bool {
        context
            .project_config
            .crates
            .iter()
            .any(|config| config.name == member.name && config.role == Some(Role::Tooling))
    }

    fn suffix(member: &Member) -> Option<String> {
        member
            .directory
            .split_once('/')
            .map(|(_, path)| path)
            .filter(|path| !path.is_empty())
            .map(|path| path.replace('/', "-"))
    }

    fn find_prefix(context: &RuleContext) -> Option<String> {
        let explicit = context
            .workspace
            .root_manifest
            .root
            .find(&["workspace", "metadata", "klibs-rules", "prefix"])
            .and_then(TomlValue::as_str);
        if let Some(prefix) = explicit {
            return Some(prefix.to_owned());
        }
        let mut candidates = BTreeMap::<String, usize>::new();
        for member in &context.workspace.members {
            if Self::is_tooling(context, member) {
                continue;
            }
            let Some(suffix) = Self::suffix(member) else {
                continue;
            };
            let suffix = format!("-{suffix}");
            if let Some(prefix) = member
                .name
                .strip_suffix(&suffix)
                .filter(|prefix| !prefix.is_empty())
            {
                *candidates.entry(prefix.to_owned()).or_default() += 1;
            }
        }
        let maximum = candidates.values().copied().max()?;
        let mut winners = candidates
            .into_iter()
            .filter(|(_, count)| *count == maximum);
        let winner = winners.next()?.0;
        winners.next().is_none().then_some(winner)
    }

    fn build_finding(member: &Member, line: Option<u32>, message: String) -> RuleFinding {
        RuleFinding {
            finding: Finding {
                rule_id: RuleId("RootPackagePath".to_owned()),
                location: Location {
                    path: member.manifest.path.clone(),
                    line,
                },
                subject: member.name.clone(),
                message,
            },
            anchor: Anchor::Crate(member.name.clone()),
        }
    }
}

impl Rule for RootPackagePath {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let Some(prefix) = Self::find_prefix(context) else {
            return vec![RuleFinding {
                finding: Finding {
                    rule_id: RuleId("RootPackagePath".to_owned()),
                    location: Location { path: context.workspace.root_manifest.path.clone(), line: None },
                    subject: ".".to_owned(),
                    message: "set `[workspace.metadata.klibs-rules] prefix`: member names give no unique prefix".to_owned(),
                },
                anchor: Anchor::Workspace,
            }];
        };
        let mut findings = Vec::new();
        for member in &context.workspace.members {
            let normalized = member.name.replace('-', "_");
            if ["std", "core", "alloc", "proc_macro", "test"].contains(&normalized.as_str()) {
                findings.push(Self::build_finding(
                    member,
                    member
                        .manifest
                        .root
                        .find(&["package", "name"])
                        .map(|value| value.line),
                    "crate names must not shadow Rust's standard crates".to_owned(),
                ));
            }
            if let Some(name) = member.manifest.root.find(&["lib", "name"]) {
                findings.push(Self::build_finding(
                    member,
                    Some(name.line),
                    "do not set `[lib] name`; the crate import name follows its package name"
                        .to_owned(),
                ));
            }
            if Self::is_tooling(context, member) {
                continue;
            }
            let Some(suffix) = Self::suffix(member) else {
                continue;
            };
            let expected = format!("{prefix}-{suffix}");
            if member.name != expected {
                findings.push(Self::build_finding(
                    member,
                    member
                        .manifest
                        .root
                        .find(&["package", "name"])
                        .map(|value| value.line),
                    format!("name this crate `{expected}` to match its unit path"),
                ));
            }
        }
        findings
    }
}
