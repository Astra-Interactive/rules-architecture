use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::config::model::Role;
use crate::dependency::util::build_dependency_finding;
use crate::workspace::model::DependencyKind;

pub(crate) struct ApiUnitDependency;

impl Rule for ApiUnitDependency {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context.workspace.members.iter().filter(|member| context.find_role_config(&member.name).and_then(|config| config.role) == Some(Role::Api)).flat_map(|member| member.dependencies.iter().filter_map(move |dependency| {
            if dependency.kind == DependencyKind::Development { return None; }
            let provider = dependency.find_member(&context.workspace.members)?;
            (!context.is_role_dependency_allowed(member, dependency, provider)).then(|| build_dependency_finding("ApiUnitDependency", member, dependency, &provider.name, "an api unit uses only other api, platform-free core and storage contracts, plus the documented macro/contract allowances"))
        })).collect()
    }
}
