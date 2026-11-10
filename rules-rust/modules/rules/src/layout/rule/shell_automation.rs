use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use std::fs;

pub(crate) struct ShellAutomation;

impl ShellAutomation {
    fn is_script(name: &str) -> bool {
        Self::SCRIPT_NAMES.contains(&name)
            || Self::SCRIPT_EXTENSIONS
                .iter()
                .any(|extension| name.ends_with(extension))
    }

    fn is_cargo_command(segment: &str) -> bool {
        segment
            .trim()
            .trim_start_matches(['@', '-', '+'])
            .split_whitespace()
            .find(|token| !Self::COMMAND_WRAPPERS.contains(token) && !token.contains('='))
            == Some("cargo")
    }

    fn runs_cargo(content: &str) -> bool {
        content
            .lines()
            .filter(|line| !line.trim_start().starts_with('#'))
            .flat_map(|line| line.split(['&', '|', ';', '(', ')', '`']))
            .any(Self::is_cargo_command)
    }

    fn is_cargo_script(context: &RuleContext, path: &str) -> bool {
        fs::read_to_string(context.workspace.root.join(path))
            .is_ok_and(|content| Self::runs_cargo(&content))
    }

    const SCRIPT_EXTENSIONS: [&str; 6] = [".sh", ".bash", ".zsh", ".ps1", ".bat", ".cmd"];
    const SCRIPT_NAMES: [&str; 5] = [
        "Makefile",
        "makefile",
        "GNUmakefile",
        "justfile",
        "Justfile",
    ];
    const COMMAND_WRAPPERS: [&str; 4] = ["exec", "sudo", "time", "env"];
}

impl Rule for ShellAutomation {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .workspace
            .files
            .iter()
            .filter_map(|path| {
                let (directory, name) = path.rsplit_once('/').unwrap_or(("", path));
                let direct = directory.is_empty()
                    || context
                        .workspace
                        .members
                        .iter()
                        .any(|member| member.directory == directory);
                if !Self::is_script(name) || !(direct || Self::is_cargo_script(context, path)) {
                    return None;
                }
                Some(RuleFinding {
                    finding: Finding {
                        rule_id: RuleId("ShellAutomation".to_owned()),
                        location: Location {
                            path: path.clone(),
                            line: None,
                        },
                        subject: path.clone(),
                        message: "build automation belongs in the xtask crate".to_owned(),
                    },
                    anchor: Anchor::Workspace,
                })
            })
            .collect()
    }
}
