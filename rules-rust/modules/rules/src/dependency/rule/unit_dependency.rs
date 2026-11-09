use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::dependency::util::build_dependency_finding;

pub(crate) struct UnitDependency;

impl Rule for UnitDependency {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context.workspace.members.iter().flat_map(|member| member.dependencies.iter().filter_map(move |dependency| {
            let provider = dependency.find_member(&context.workspace.members)?;
            (!context.is_role_dependency_allowed(member, dependency, provider)).then(|| build_dependency_finding("UnitDependency", member, dependency, &provider.name, "the workspace dependency is forbidden by the dependent role, fake-feature or tooling/proc-macro boundary"))
        })).collect()
    }
}
