use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::config::model::Role;

pub(crate) struct PluginImplUnit;

impl Rule for PluginImplUnit {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        if !context.is_plugin_product() {
            return Vec::new();
        }
        let api_parents: Vec<_> = context
            .workspace
            .members
            .iter()
            .filter(|member| {
                context
                    .find_role_config(&member.name)
                    .and_then(|config| config.role)
                    == Some(Role::Api)
            })
            .filter_map(|member| member.directory.rsplit_once('/').map(|(parent, _)| parent))
            .collect();
        context.workspace.members.iter().filter_map(|member| {
            let role = context.find_role_config(&member.name).and_then(|config| config.role);
            let sibling = member.directory.rsplit_once('/').is_some_and(|(parent, _)| api_parents.contains(&parent));
            let is_allowed_sibling = matches!(role, Some(Role::Api | Role::Platform | Role::Technology | Role::ProcMacro));
            let forbidden = matches!(role, Some(Role::Impl | Role::ImplUi | Role::Core)) || (sibling && !is_allowed_sibling);
            forbidden.then(|| RuleFinding {
                finding: Finding { rule_id: RuleId("PluginImplUnit".to_owned()), location: Location { path: member.manifest.path.clone(), line: None }, subject: member.name.clone(), message: "a plugin sub-context is unsplit or api plus platform units; no impl/impl-ui sibling".to_owned() },
                anchor: Anchor::Crate(member.name.clone()),
            })
        }).collect()
    }
}
