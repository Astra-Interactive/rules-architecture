use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::config::model::Role;
use crate::dependency::util::build_dependency_finding;
use crate::workspace::model::DependencyKind;

pub(crate) struct ApiUnitPlatformDependency;

impl Rule for ApiUnitPlatformDependency {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let words = &context.project_config.platform_words;
        if words.is_empty() {
            return Vec::new();
        }
        context.workspace.members.iter().flat_map(|member| member.dependencies.iter().filter_map(move |dependency| {
            let name = dependency.name.replace('-', "_");
            if !words.iter().any(|word| RuleContext::matches_crate_word(&name, word)) { return None; }
            let role = context.find_role_config(&member.name).and_then(|config| config.role);
            let forbidden = match role {
                Some(Role::Api) => context.is_plugin_product() || dependency.kind != DependencyKind::Normal || !dependency.is_target_specific,
                Some(Role::CoreApi | Role::StorageContract | Role::Contract) => true,
                _ => false,
            };
            forbidden.then(|| build_dependency_finding("ApiUnitPlatformDependency", member, dependency, &dependency.name, "platform-free units declare no platform dependency; an app api allows one only in target cfg dependencies"))
        })).collect()
    }
}
