use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::config::model::Role;

pub(crate) struct UnitPath;

impl UnitPath {
    fn build_finding(name: &str, path: &str, message: &str) -> RuleFinding {
        RuleFinding {
            finding: Finding {
                rule_id: RuleId("UnitPath".to_owned()),
                location: Location {
                    path: if path.is_empty() {
                        ".".to_owned()
                    } else {
                        path.to_owned()
                    },
                    line: None,
                },
                subject: name.to_owned(),
                message: message.to_owned(),
            },
            anchor: Anchor::Crate(name.to_owned()),
        }
    }
}

impl Rule for UnitPath {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for member in &context.workspace.members {
            let role = context
                .project_config
                .crates
                .iter()
                .find(|config| config.name == member.name)
                .and_then(|config| config.role);
            let segments: Vec<&str> = member.directory.split('/').collect();
            let valid = match role {
                Some(Role::EntryPoint) => segments.len() == 2 && segments[0] == "instances",
                _ => (2..=4).contains(&segments.len()) && segments[0] == "modules",
            };
            if !valid || segments.contains(&"shared") {
                findings.push(Self::build_finding(&member.name, &member.directory,
                    "place entry points at instances/<name> and product crates at modules/<a>[/<b>[/<c>]]; no segment is shared"));
            }
        }
        findings
    }
}
