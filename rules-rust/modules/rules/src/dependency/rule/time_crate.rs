use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::workspace::model::DependencyKind;
use std::collections::BTreeSet;

pub(crate) struct TimeCrate;

impl Rule for TimeCrate {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let clocks: BTreeSet<&str> = context
            .workspace
            .members
            .iter()
            .flat_map(|member| &member.dependencies)
            .filter(|dependency| dependency.kind != DependencyKind::Development)
            .map(|dependency| dependency.name.as_str())
            .filter(|name| ["chrono", "time", "jiff"].contains(name))
            .collect();
        if clocks.len() <= 1 {
            return Vec::new();
        }
        vec![RuleFinding {
            finding: Finding {
                rule_id: RuleId("TimeCrate".to_owned()),
                location: Location {
                    path: context.workspace.root_manifest.path.clone(),
                    line: None,
                },
                subject: ".".to_owned(),
                message: format!(
                    "choose one non-development time crate for the workspace; found {}",
                    clocks.into_iter().collect::<Vec<_>>().join(", ")
                ),
            },
            anchor: Anchor::Workspace,
        }]
    }
}
