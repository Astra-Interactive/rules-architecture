use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::workspace::model::TomlValue;

pub(crate) struct ProjectDirMatchesPath;

impl Rule for ProjectDirMatchesPath {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let manifest = &context.workspace.root_manifest;
        manifest
            .root
            .find(&["workspace", "members"])
            .and_then(TomlValue::as_array)
            .into_iter()
            .flatten()
            .filter_map(|entry| {
                let path = entry.as_str()?;
                if !path.contains(['*', '?', '[']) {
                    return None;
                }
                Some(RuleFinding {
                    finding: Finding {
                        rule_id: RuleId("ProjectDirMatchesPath".to_owned()),
                        location: Location {
                            path: manifest.path.clone(),
                            line: Some(entry.line),
                        },
                        subject: path.to_owned(),
                        message: "list each member path explicitly: workspace members use no glob"
                            .to_owned(),
                    },
                    anchor: Anchor::Workspace,
                })
            })
            .collect()
    }
}
