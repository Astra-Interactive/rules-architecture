use crate::check::api::Rule;
use crate::layout::rule::ContainerSources;
use crate::test::check::usecase::read_case_context;

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
