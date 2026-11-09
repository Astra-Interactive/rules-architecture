use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::config::model::Role;
use crate::dependency::util::build_dependency_finding;

pub(crate) struct TechnologyDependency;

impl Rule for TechnologyDependency {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let owners: Vec<_> = context
            .workspace
            .members
            .iter()
            .filter(|member| {
                context
                    .find_role_config(&member.name)
                    .and_then(|config| config.role)
                    == Some(Role::Technology)
            })
            .filter_map(|member| {
                member
                    .directory
                    .rsplit('/')
                    .next()
                    .map(|word| (member.name.as_str(), word))
            })
            .collect();
        context.workspace.members.iter().filter(|member| context.find_role_config(&member.name).and_then(|config| config.role) != Some(Role::EntryPoint)).flat_map(|member| {
            let owners = &owners;
            member.dependencies.iter().filter_map(move |dependency| {
                if dependency.find_member(&context.workspace.members).is_some() { return None; }
                let full = dependency.name.replace('-', "_");
                if owners.iter().any(|(owner, word)| *owner == member.name && RuleContext::matches_crate_word(&full, word)) { return None; }
                let other_owner = owners.iter().any(|(owner, word)| *owner != member.name && RuleContext::matches_crate_word(&full, word));
                other_owner.then(|| build_dependency_finding("TechnologyDependency", member, dependency, &dependency.name, "an external technology owned by a workspace technology unit is used only there or by entry points"))
            }).collect::<Vec<_>>()
        }).collect()
    }
}
