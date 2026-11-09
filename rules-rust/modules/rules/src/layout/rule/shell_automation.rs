use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};

pub(crate) struct ShellAutomation;

impl Rule for ShellAutomation {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .workspace
            .files
            .iter()
            .filter_map(|path| {
                let (directory, name) = path.rsplit_once('/').unwrap_or(("", path));
                let direct = directory.is_empty()
                    || context
                        .workspace
                        .members
                        .iter()
                        .any(|member| member.directory == directory);
                let script = name.ends_with(".sh")
                    || ["Makefile", "makefile", "justfile", "Justfile"].contains(&name);
                if !direct || !script {
                    return None;
                }
                Some(RuleFinding {
                    finding: Finding {
                        rule_id: RuleId("ShellAutomation".to_owned()),
                        location: Location {
                            path: path.clone(),
                            line: None,
                        },
                        subject: path.clone(),
                        message: "build automation belongs in the xtask crate".to_owned(),
                    },
                    anchor: Anchor::Workspace,
                })
            })
            .collect()
    }
}
