use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::config::model::Role;
use crate::workspace::model::TomlValue;

pub(crate) struct XtaskAlias;

impl XtaskAlias {
    fn format_alias(value: &TomlValue) -> Option<String> {
        if let Some(text) = value.as_str() {
            return Some(text.to_owned());
        }
        let words = value
            .as_array()?
            .iter()
            .map(TomlValue::as_str)
            .collect::<Option<Vec<&str>>>()?;
        Some(words.join(" "))
    }
}

impl Rule for XtaskAlias {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let accepted: Vec<String> = context
            .workspace
            .members
            .iter()
            .filter(|member| {
                member.name == "xtask"
                    || context
                        .find_role_config(&member.name)
                        .and_then(|config| config.role)
                        == Some(Role::Tooling)
            })
            .map(|member| format!("run --package {} --", member.name))
            .collect();
        if accepted.is_empty() {
            return Vec::new();
        }
        let config = context.workspace.cargo_config.as_ref();
        let alias = config.and_then(|config| config.root.find(&["alias", "xtask"]));
        if alias
            .and_then(Self::format_alias)
            .is_some_and(|value| accepted.contains(&value))
        {
            return Vec::new();
        }
        let path = config.map_or(".cargo/config.toml", |config| config.path.as_str());
        vec![RuleFinding {
            finding: Finding {
                rule_id: RuleId("XtaskAlias".to_owned()),
                location: Location { path: path.to_owned(), line: alias.map(|value| value.line) },
                subject: path.to_owned(),
                message: "set `[alias] xtask = \"run --package xtask --\"` in the root Cargo configuration".to_owned(),
            },
            anchor: Anchor::Workspace,
        }]
    }
}
