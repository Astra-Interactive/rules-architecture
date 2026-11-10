use crate::check::api::Rule;
use crate::layout::rule::ShellAutomation;
use crate::test::check::usecase::read_case_context;
use crate::workspace::api::WorkspaceReader;
use crate::workspace::cargo::CargoWorkspaceReader;
use crate::workspace::util::resolve_cargo_path;
use std::fs;
use std::path::Path;

fn write_file(root: &Path, path: &str, content: &str) {
    let file = root.join(path);
    fs::create_dir_all(file.parent().expect("file parent")).expect("directory");
    fs::write(file, content).expect("file");
}

#[test]
fn given_policy_fixtures_when_checked_then_incorrect_is_detected_and_correct_is_accepted() {
    let fail = read_case_context("ShellAutomation", "fail-policy");
    assert!(!ShellAutomation.check(&fail).is_empty());
    let pass = read_case_context("ShellAutomation", "pass-policy");
    assert!(ShellAutomation.check(&pass).is_empty());
}

#[test]
fn given_automation_paths_when_checked_then_only_direct_root_and_member_scripts_are_reported() {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    context.workspace.files = [
        "build.sh",
        "Makefile",
        "makefile",
        "Justfile",
        "justfile",
        "modules/sample/run.sh",
        "scripts/build.sh",
        "modules/sample/scripts/run.sh",
        "modules/Makefile",
        "build.SH",
    ]
    .map(str::to_owned)
    .to_vec();
    let findings = ShellAutomation.check(&context);
    let subjects: Vec<&str> = findings
        .iter()
        .map(|finding| finding.finding.subject.as_str())
        .collect();
    assert_eq!(
        subjects,
        [
            "build.sh",
            "Makefile",
            "makefile",
            "Justfile",
            "justfile",
            "modules/sample/run.sh"
        ]
    );
    for finding in findings {
        assert_eq!(finding.finding.location.path, finding.finding.subject);
        assert_eq!(finding.finding.location.line, None);
    }
    context.workspace.files = [
        "scripts/build.sh",
        "modules/sample/scripts/run.sh",
        "modules/Makefile",
        "build.SH",
    ]
    .map(str::to_owned)
    .to_vec();
    assert!(ShellAutomation.check(&context).is_empty());
}

#[test]
fn given_nested_script_that_builds_the_workspace_when_checked_then_only_that_script_is_reported() {
    let root = tempfile::tempdir().expect("workspace directory");
    write_file(
        root.path(),
        "Cargo.toml",
        "[workspace]\nresolver = \"3\"\nmembers = [\"modules/sample\"]\n",
    );
    write_file(
        root.path(),
        "modules/sample/Cargo.toml",
        "[package]\nname = \"case-sample\"\nedition = \"2024\"\n",
    );
    write_file(root.path(), "modules/sample/src/lib.rs", "");
    write_file(
        root.path(),
        "scripts/build.sh",
        "#!/bin/sh\ncargo build --workspace\n",
    );
    write_file(
        root.path(),
        "scripts/serve.sh",
        "#!/bin/sh\nexec ./server --port 8080\n",
    );
    let mut context = read_case_context("RootFileShape", "pass-tree");
    context.workspace = CargoWorkspaceReader::new(resolve_cargo_path())
        .read_workspace(root.path())
        .expect("Cargo-valid workspace");
    let findings = ShellAutomation.check(&context);
    let subjects: Vec<&str> = findings
        .iter()
        .map(|finding| finding.finding.subject.as_str())
        .collect();
    assert_eq!(subjects, ["scripts/build.sh"]);
    assert_eq!(findings[0].finding.rule_id.0, "ShellAutomation");
}

#[test]
fn given_other_script_spellings_at_the_root_when_checked_then_each_is_reported() {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    context.workspace.files = ["release.bash", "build.ps1", "GNUmakefile", "README.md"]
        .map(str::to_owned)
        .to_vec();
    let subjects: Vec<String> = ShellAutomation
        .check(&context)
        .into_iter()
        .map(|finding| finding.finding.subject)
        .collect();
    assert_eq!(subjects, ["release.bash", "build.ps1", "GNUmakefile"]);
}
