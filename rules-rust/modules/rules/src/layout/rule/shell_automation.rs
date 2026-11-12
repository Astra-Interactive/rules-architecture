use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use std::fs;
use std::io::{BufRead, BufReader, Read};

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
            .find(|token| {
                !Self::COMMAND_WRAPPERS.contains(token)
                    && !Self::SHELL_KEYWORDS.contains(token)
                    && !token.contains('=')
            })
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

    fn is_shell_shebang(line: &str) -> bool {
        let Some(command) = line.trim().strip_prefix("#!") else {
            return false;
        };
        let program_name = |word: &str| word.rsplit('/').next().unwrap_or(word).to_owned();
        let mut words = command.split_whitespace();
        let Some(program) = words.next().map(program_name) else {
            return false;
        };
        let interpreter = if program == "env" {
            words
                .find(|word| !word.starts_with('-') && !word.contains('='))
                .map(program_name)
        } else {
            Some(program)
        };
        interpreter.is_some_and(|name| Self::SHELL_INTERPRETERS.contains(&name.as_str()))
    }

    fn is_shell_script_without_extension(context: &RuleContext, path: &str) -> bool {
        let Ok(file) = fs::File::open(context.workspace.root.join(path)) else {
            return false;
        };
        let mut first_line = String::new();
        BufReader::new(file.take(Self::SHEBANG_LIMIT))
            .read_line(&mut first_line)
            .is_ok()
            && Self::is_shell_shebang(&first_line)
    }

    const SHEBANG_LIMIT: u64 = 256;
    const SHELL_INTERPRETERS: [&str; 9] = [
        "sh", "bash", "zsh", "dash", "ash", "ksh", "fish", "csh", "tcsh",
    ];
    const SCRIPT_EXTENSIONS: [&str; 9] = [
        ".sh", ".bash", ".zsh", ".ksh", ".fish", ".csh", ".ps1", ".bat", ".cmd",
    ];
    const SCRIPT_NAMES: [&str; 10] = [
        "Makefile",
        "makefile",
        "GNUmakefile",
        "justfile",
        "Justfile",
        ".justfile",
        ".Justfile",
        "Makefile.toml",
        "Taskfile.yml",
        "Taskfile.yaml",
    ];
    const COMMAND_WRAPPERS: [&str; 4] = ["exec", "sudo", "time", "env"];
    const SHELL_KEYWORDS: [&str; 10] = [
        "if", "elif", "while", "until", "then", "do", "else", "!", "{", "}",
    ];
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
                let script =
                    Self::is_script(name) || Self::is_shell_script_without_extension(context, path);
                if !script || !(direct || Self::is_cargo_script(context, path)) {
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
