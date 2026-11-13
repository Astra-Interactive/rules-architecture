use crate::check::api::Rule;
use crate::readme::rule::UnitReadme;
use crate::test::check::usecase::read_case_context;
use crate::workspace::api::WorkspaceReader;
use crate::workspace::cargo::CargoWorkspaceReader;
use crate::workspace::util::parse_toml_file;
use crate::workspace::util::resolve_cargo_path;
use std::fs;
use std::path::Path;

fn write_file(root: &Path, path: &str, content: &str) {
    let file = root.join(path);
    fs::create_dir_all(file.parent().expect("file parent")).expect("directory");
    fs::write(file, content).expect("file");
}

#[test]
fn given_alternate_readme_path_when_checked_then_the_manifest_value_is_reported() {
    let mut context = read_case_context("UnitReadme", "pass-one-sentence");
    context.workspace.members[0].manifest =
        parse_toml_file("Cargo.toml", "[package]\nreadme=\"OTHER.md\"\n").expect("TOML");
    let findings = UnitReadme.check(&context);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].finding.message.contains("OTHER.md"));
    context.workspace.members[0].manifest =
        parse_toml_file("Cargo.toml", "[package]\nreadme=\"README.md\"\n").expect("TOML");
    assert!(UnitReadme.check(&context).is_empty());
    context.workspace.members[0].manifest =
        parse_toml_file("Cargo.toml", "[package]\nreadme=true\n").expect("TOML");
    assert!(UnitReadme.check(&context).is_empty());
}

#[test]
fn given_inherited_readme_when_the_root_readme_has_two_sentences_then_the_member_is_reported() {
    let root = tempfile::tempdir().expect("workspace directory");
    write_file(
        root.path(),
        "Cargo.toml",
        "[workspace]\nresolver = \"3\"\nmembers = [\"modules/sample\"]\n\n[workspace.package]\nreadme = \"README.md\"\n",
    );
    write_file(
        root.path(),
        "README.md",
        "Holds the shop crates. Used by the host.\n",
    );
    write_file(
        root.path(),
        "modules/sample/Cargo.toml",
        "[package]\nname = \"case-sample\"\nedition = \"2024\"\nreadme.workspace = true\n",
    );
    write_file(root.path(), "modules/sample/src/lib.rs", "");
    write_file(
        root.path(),
        "modules/sample/README.md",
        "Holds the sample responsibility.\n",
    );
    let mut context = read_case_context("UnitReadme", "pass-one-sentence");
    context.workspace = CargoWorkspaceReader::new(resolve_cargo_path())
        .read_workspace(root.path())
        .expect("Cargo-valid workspace");
    let findings = UnitReadme.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.rule_id.0, "UnitReadme");
    assert_eq!(findings[0].finding.subject, "case-sample");
}

#[test]
fn given_inherited_readme_and_no_readme_of_its_own_when_checked_then_the_missing_file_reports() {
    let root = tempfile::tempdir().expect("workspace directory");
    write_file(
        root.path(),
        "Cargo.toml",
        "[workspace]\nresolver = \"3\"\nmembers = [\"modules/sample\"]\n\n[workspace.package]\nreadme = \"README.md\"\n",
    );
    write_file(root.path(), "README.md", "Holds the shop crates.\n");
    write_file(
        root.path(),
        "modules/sample/Cargo.toml",
        "[package]\nname = \"case-sample\"\nedition = \"2024\"\nreadme.workspace = true\n",
    );
    write_file(root.path(), "modules/sample/src/lib.rs", "");
    let mut context = read_case_context("UnitReadme", "pass-one-sentence");
    context.workspace = CargoWorkspaceReader::new(resolve_cargo_path())
        .read_workspace(root.path())
        .expect("Cargo-valid workspace");
    let messages: Vec<String> = UnitReadme
        .check(&context)
        .into_iter()
        .map(|finding| finding.finding.message)
        .collect();
    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(messages[0].contains("Add a README.md"), "{messages:?}");
}

#[test]
fn given_unterminated_second_sentence_when_checked_then_both_sentences_are_counted() {
    let mut context = read_case_context("UnitReadme", "pass-one-sentence");
    context.workspace.members[0].readme =
        Some("Holds the billing contracts. Used by the shop crates\n".to_owned());
    let findings = UnitReadme.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.rule_id.0, "UnitReadme");
    assert_eq!(
        findings[0].finding.subject,
        context.workspace.members[0].name
    );
    assert!(findings[0].finding.message.contains("2 sentences"));
}
