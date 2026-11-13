use crate::check::api::Rule;
use crate::check::model::Anchor;
use crate::check::model::Finding;
use crate::check::model::Location;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::model::RuleId;

pub(crate) struct RootSources;

impl RootSources {
    fn build_finding(location: Location, message: String) -> RuleFinding {
        RuleFinding {
            finding: Finding {
                rule_id: RuleId(Self::RULE.to_owned()),
                location,
                subject: ".".to_owned(),
                message,
            },
            anchor: Anchor::Workspace,
        }
    }

    const RULE: &str = "RootSources";
}

impl Rule for RootSources {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let workspace = &context.workspace;
        let mut findings = Vec::new();
        if let Some(package) = workspace.root_manifest.root.find_entry("package") {
            let location = Location {
                path: workspace.root_manifest.path.clone(),
                line: Some(package.line),
            };
            let message = "the root manifest is virtual: move the crate to modules/<name> and keep only [workspace] here".to_owned();
            findings.push(Self::build_finding(location, message));
        }
        let has_src_file = workspace.files.iter().any(|file| file.starts_with("src/"));
        let has_src_directory = workspace
            .directories
            .iter()
            .any(|directory| directory == "src" || directory.starts_with("src/"));
        if has_src_file || has_src_directory {
            let location = Location {
                path: "src".to_owned(),
                line: None,
            };
            let message = "the workspace root holds no src/: every crate sits in its own directory"
                .to_owned();
            findings.push(Self::build_finding(location, message));
        }
        findings
    }
}
