use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use std::collections::BTreeSet;

pub(crate) struct ContainerSources;

impl Rule for ContainerSources {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let workspace = &context.workspace;
        let containers: BTreeSet<&str> = workspace
            .members
            .iter()
            .flat_map(|member| {
                member
                    .directory
                    .match_indices('/')
                    .map(|(index, _)| &member.directory[..index])
            })
            .collect();
        containers.into_iter().filter(|directory| {
            let has_sources = workspace.directories.iter().any(|path| path == &format!("{directory}/src"));
            let has_file = workspace.files.iter().any(|path| {
                let Some(file) = path.strip_prefix(&format!("{directory}/")) else { return false; };
                !file.contains('/') && file != "README.md"
            });
            has_sources || has_file
        }).map(|directory| RuleFinding {
            finding: Finding {
                rule_id: RuleId("ContainerSources".to_owned()),
                location: Location { path: directory.to_owned(), line: None },
                subject: directory.to_owned(),
                message: "a container holds crate directories only: move Cargo.toml and sources into a member".to_owned(),
            },
            anchor: Anchor::Workspace,
        }).collect()
    }
}
