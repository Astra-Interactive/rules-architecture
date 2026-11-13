use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::config::model::Role;
use crate::workspace::model::Target;

pub(crate) struct EntryPointDependency;

impl Rule for EntryPointDependency {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for member in &context.workspace.members {
            for dependency in &member.dependencies {
                let Some(provider) = dependency.find_member(&context.workspace.members) else {
                    continue;
                };
                let role = context
                    .project_config
                    .crates
                    .iter()
                    .find(|config| config.name == provider.name)
                    .and_then(|config| config.role);
                let has_library = provider.targets.iter().any(Target::is_rust_library);
                if role != Some(Role::EntryPoint) && has_library {
                    continue;
                }
                findings.push(RuleFinding {
                    finding: Finding {
                        rule_id: RuleId("EntryPointDependency".to_owned()),
                        location: Location { path: member.manifest.path.clone(), line: dependency.line },
                        subject: format!("{} -> {}", member.name, provider.name),
                        message: "nothing depends on an entry-point crate or a crate without a Rust library target".to_owned(),
                    },
                    anchor: Anchor::Unexpectable,
                });
            }
        }
        findings
    }
}
