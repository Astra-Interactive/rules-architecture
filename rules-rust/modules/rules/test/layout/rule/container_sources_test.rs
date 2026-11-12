use crate::check::api::Rule;
use crate::layout::rule::ContainerSources;
use crate::test::check::usecase::read_case_context;
use crate::workspace::api::WorkspaceReader;
use crate::workspace::cargo::CargoWorkspaceReader;
use crate::workspace::util::resolve_cargo_path;
use std::fs;

#[test]
fn given_container_content_when_checked_then_manifest_source_directory_and_rust_files_are_reported()
{
    let context = read_case_context("ContainerSources", "fail-container-content");
    let subjects: Vec<String> = ContainerSources
        .check(&context)
        .into_iter()
        .map(|finding| finding.finding.subject)
        .collect();
    assert_eq!(
        subjects,
        ["modules/loose", "modules/manifest", "modules/source"]
    );
    let context = read_case_context("ContainerSources", "pass-empty-containers");
    assert!(ContainerSources.check(&context).is_empty());
}

#[test]
fn given_non_rust_file_beside_a_crate_directory_when_checked_then_the_container_is_reported() {
    let mut context = read_case_context("ContainerSources", "pass-empty-containers");
    context
        .workspace
        .files
        .push("modules/rating/notes.txt".to_owned());
    let findings = ContainerSources.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.rule_id.0, "ContainerSources");
    assert_eq!(findings[0].finding.subject, "modules/rating");
}

#[test]
fn given_stray_child_directory_and_nested_container_when_checked_then_only_the_stray_one_reports() {
    let mut context = read_case_context("ContainerSources", "pass-empty-containers");
    let mut storage = context.workspace.members[0].clone();
    storage.name = "billing-storage-sqlx".to_owned();
    storage.directory = "modules/billing/storage/sqlx".to_owned();
    context.workspace.members.push(storage);
    context.workspace.files.extend(
        [
            "modules/billing/storage/sqlx/Cargo.toml",
            "modules/billing/storage/sqlx/src/lib.rs",
            "modules/rating/notes/readme.txt",
        ]
        .map(str::to_owned),
    );
    let findings = ContainerSources.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].finding.rule_id.0, "ContainerSources");
    assert_eq!(findings[0].finding.subject, "modules/rating");
}

#[test]
fn given_file_in_a_child_directory_without_a_member_when_checked_then_the_container_is_reported() {
    let mut context = read_case_context("ContainerSources", "pass-empty-containers");
    context.workspace.files.extend(
        [
            "modules/rating/README.md",
            "modules/rating/api/docs/guide.md",
            "modules/rating/docs/design.md",
        ]
        .map(str::to_owned),
    );
    let findings = ContainerSources.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].finding.rule_id.0, "ContainerSources");
    assert_eq!(findings[0].finding.subject, "modules/rating");
}

#[test]
fn given_skipped_directory_inside_a_container_when_checked_then_the_container_is_reported() {
    let mut context = read_case_context("ContainerSources", "pass-empty-containers");
    let clean = context.workspace.directories.clone();
    context
        .workspace
        .directories
        .push("modules/rating/resources".to_owned());
    let findings = ContainerSources.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].finding.subject, "modules/rating");
    context.workspace.directories = clean;
    context
        .workspace
        .directories
        .push("modules/rating/api/resources".to_owned());
    assert!(ContainerSources.check(&context).is_empty());
}

#[test]
fn given_resources_directory_in_a_container_when_the_workspace_is_read_then_it_is_recorded_unwalked()
 {
    let root = tempfile::tempdir().expect("workspace directory");
    for (path, content) in [
        (
            "Cargo.toml",
            "[workspace]\nresolver = \"3\"\nmembers = [\"modules/rating/api\"]\n",
        ),
        (
            "modules/rating/api/Cargo.toml",
            "[package]\nname = \"case-rating-api\"\nedition = \"2024\"\n",
        ),
        ("modules/rating/api/src/lib.rs", ""),
        ("modules/rating/resources/icon.png", "png"),
    ] {
        let file = root.path().join(path);
        fs::create_dir_all(file.parent().expect("file parent")).expect("directory");
        fs::write(file, content).expect("file");
    }
    let workspace = CargoWorkspaceReader::new(resolve_cargo_path())
        .read_workspace(root.path())
        .expect("Cargo-valid workspace");
    assert!(
        workspace
            .directories
            .iter()
            .any(|directory| directory == "modules/rating/resources"),
        "{:?}",
        workspace.directories
    );
    assert!(
        !workspace
            .files
            .iter()
            .any(|file| file == "modules/rating/resources/icon.png")
    );
}
