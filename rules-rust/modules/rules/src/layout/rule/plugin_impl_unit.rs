use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::config::model::Role;
use crate::workspace::model::Member;

pub(crate) struct PluginImplUnit;

impl PluginImplUnit {
    fn find_parent(member: &Member) -> Option<&str> {
        member.directory.rsplit_once('/').map(|(parent, _)| parent)
    }

    fn find_role(context: &RuleContext, member: &Member) -> Option<Role> {
        context
            .find_role_config(&member.name)
            .and_then(|config| config.role)
    }

    fn is_split_without_contract(context: &RuleContext, parent: &str) -> bool {
        if !parent.contains('/') {
            return false;
        }
        let siblings: Vec<&Member> = context
            .workspace
            .members
            .iter()
            .filter(|member| Self::find_parent(member) == Some(parent))
            .collect();
        let has_contract = siblings.iter().any(|member| {
            matches!(
                Self::find_role(context, member),
                Some(Role::Api | Role::Contract | Role::StorageContract)
            )
        });
        let context_units = siblings
            .iter()
            .filter(|member| Self::find_role(context, member) == Some(Role::Context))
            .count();
        context_units >= 2 && !has_contract
    }
}

impl Rule for PluginImplUnit {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        if !context.is_plugin_product() {
            return Vec::new();
        }
        let api_parents: Vec<_> = context
            .workspace
            .members
            .iter()
            .filter(|member| Self::find_role(context, member) == Some(Role::Api))
            .filter_map(Self::find_parent)
            .collect();
        context.workspace.members.iter().filter_map(|member| {
            let role = Self::find_role(context, member);
            let parent = Self::find_parent(member);
            let sibling = parent.is_some_and(|parent| api_parents.contains(&parent));
            let is_allowed_sibling = matches!(role, Some(Role::Api | Role::Platform | Role::Technology | Role::ProcMacro));
            let is_context_unit = role == Some(Role::Context);
            let split_without_contract = is_context_unit && parent.is_some_and(|parent| Self::is_split_without_contract(context, parent));
            let forbidden = matches!(role, Some(Role::Impl | Role::ImplUi | Role::Core)) || (sibling && !is_allowed_sibling) || split_without_contract;
            forbidden.then(|| RuleFinding {
                finding: Finding { rule_id: RuleId("PluginImplUnit".to_owned()), location: Location { path: member.manifest.path.clone(), line: None }, subject: member.name.clone(), message: "a plugin sub-context is unsplit or api plus platform units; no impl/impl-ui sibling".to_owned() },
                anchor: Anchor::Crate(member.name.clone()),
            })
        }).collect()
    }
}
