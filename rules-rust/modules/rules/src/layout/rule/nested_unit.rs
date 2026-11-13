use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};

pub(crate) struct NestedUnit;

impl Rule for NestedUnit {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let members = &context.workspace.members;
        members
            .iter()
            .filter_map(|inner| {
                let outer = members.iter().find(|outer| {
                    if outer.directory.is_empty() {
                        return !inner.directory.is_empty();
                    }
                    inner
                        .directory
                        .starts_with(&format!("{}/", outer.directory))
                })?;
                Some(RuleFinding {
                    finding: Finding {
                        rule_id: RuleId("NestedUnit".to_owned()),
                        location: Location {
                            path: inner.manifest.path.clone(),
                            line: None,
                        },
                        subject: inner.name.clone(),
                        message: format!(
                            "{} is inside {}: a crate directory holds no other crate",
                            inner.directory,
                            if outer.directory.is_empty() {
                                "."
                            } else {
                                &outer.directory
                            }
                        ),
                    },
                    anchor: Anchor::Crate(inner.name.clone()),
                })
            })
            .collect()
    }
}
