use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::workspace::model::TomlValue;

pub(crate) struct PublishedVersion;

impl Rule for PublishedVersion {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for member in &context.workspace.members {
            let Some(package) = member.manifest.root.find(&["package"]) else {
                continue;
            };
            let version = package.find(&["version"]);
            let declared_publish = package.find(&["publish"]);
            let inherited = declared_publish
                .and_then(|value| value.find(&["workspace"]))
                .and_then(TomlValue::as_bool)
                == Some(true);
            let publish = if inherited {
                context
                    .workspace
                    .root_manifest
                    .root
                    .find(&["workspace", "package", "publish"])
            } else {
                declared_publish
            };
            let disabled = publish.is_some_and(|value| {
                value.as_bool() == Some(false) || value.as_array().is_some_and(<[_]>::is_empty)
            });
            let enabled = publish.is_some_and(|value| {
                value.as_bool() == Some(true)
                    || value
                        .as_array()
                        .is_some_and(|registries| !registries.is_empty())
            });
            let message = if disabled && version.is_some() {
                "an unpublished crate omits `version`"
            } else if enabled && version.is_none() {
                "a published crate declares `version` (literal or workspace-inherited)"
            } else {
                continue;
            };
            findings.push(RuleFinding {
                finding: Finding {
                    rule_id: RuleId("PublishedVersion".to_owned()),
                    location: Location {
                        path: member.manifest.path.clone(),
                        line: version
                            .or(declared_publish)
                            .map(|value: &TomlValue| value.line),
                    },
                    subject: member.name.clone(),
                    message: message.to_owned(),
                },
                anchor: Anchor::Crate(member.name.clone()),
            });
        }
        findings
    }
}
