use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::layout::rule::ProjectDirMatchesPath;
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

fn read_path_dependency_context(root: &Path, members: &str) -> RuleContext {
    write_file(
        root,
        "Cargo.toml",
        &format!("[workspace]\nresolver = \"3\"\nmembers = [{members}]\n"),
    );
    write_file(
        root,
        "modules/order/Cargo.toml",
        "[package]\nname = \"shop-order\"\nedition = \"2024\"\n[dependencies]\nshop-billing-api = { path = \"../billing/api\" }\n",
    );
    write_file(root, "modules/order/src/lib.rs", "");
    write_file(
        root,
        "modules/billing/api/Cargo.toml",
        "[package]\nname = \"shop-billing-api\"\nedition = \"2024\"\n",
    );
    write_file(root, "modules/billing/api/src/lib.rs", "");
    let mut context = read_case_context("ProjectDirMatchesPath", "pass-explicit-members");
    context.workspace = CargoWorkspaceReader::new(resolve_cargo_path())
        .read_workspace(root)
        .expect("Cargo-valid workspace");
    context
}

#[test]
fn given_member_globs_when_checked_then_each_pattern_is_reported_at_its_manifest_line() {
    let context = read_case_context("ProjectDirMatchesPath", "fail-member-globs");
    let findings = ProjectDirMatchesPath.check(&context);
    assert_eq!(findings.len(), 3);
    assert!(
        findings
            .iter()
            .all(|finding| finding.finding.location.line == Some(3))
    );
    let context = read_case_context("ProjectDirMatchesPath", "pass-explicit-members");
    assert!(ProjectDirMatchesPath.check(&context).is_empty());
}

#[test]
fn given_member_reached_only_by_path_dependency_when_checked_then_the_unlisted_member_is_reported()
{
    let root = tempfile::tempdir().expect("workspace directory");
    let context = read_path_dependency_context(root.path(), "\"modules/order\"");
    let findings = ProjectDirMatchesPath.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.rule_id.0, "ProjectDirMatchesPath");
    assert_eq!(findings[0].finding.subject, "modules/billing/api");
    assert_eq!(findings[0].finding.location.line, Some(3));
    let listed = tempfile::tempdir().expect("workspace directory");
    let context =
        read_path_dependency_context(listed.path(), "\"modules/order\", \"modules/billing/api\"");
    assert!(ProjectDirMatchesPath.check(&context).is_empty());
}
