use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::dependency::util::build_dependency_finding;

pub(crate) struct UnitDependency;

impl Rule for UnitDependency {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let members = &context.workspace.members;
        let mut findings = Vec::new();
        for member in members {
            for dependency in &member.dependencies {
                let Some(provider) = dependency.find_member(members) else {
                    continue;
                };
                let Some(message) = context.find_dependency_violation(member, dependency, provider)
                else {
                    continue;
                };
                findings.push(build_dependency_finding(
                    "UnitDependency",
                    member,
                    dependency,
                    &provider.name,
                    &message,
                ));
            }
        }
        findings
    }
}
