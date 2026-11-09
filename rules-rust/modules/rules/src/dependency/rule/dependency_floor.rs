use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::workspace::model::TomlValue;
use cargo_metadata::semver::Version;

pub(crate) struct DependencyFloor;

impl Rule for DependencyFloor {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let Some(lock) = &context.workspace.lock_file else {
            return Vec::new();
        };
        lock.root
            .find(&["package"])
            .and_then(TomlValue::as_array)
            .into_iter()
            .flatten()
            .filter_map(|package| {
                let name = package.find(&["name"])?.as_str()?;
                if name != "async-trait" {
                    return None;
                }
                let version = package.find(&["version"])?;
                let text = version.as_str()?;
                let parsed = Version::parse(text).ok()?;
                if parsed >= Version::new(0, 1, 92) {
                    return None;
                }
                Some(RuleFinding {
                    finding: Finding {
                        rule_id: RuleId("DependencyFloor".to_owned()),
                        location: Location {
                            path: lock.path.clone(),
                            line: Some(version.line),
                        },
                        subject: format!("{name} {text}"),
                        message: "async-trait must resolve to 0.1.92 or later".to_owned(),
                    },
                    anchor: Anchor::Workspace,
                })
            })
            .collect()
    }
}
