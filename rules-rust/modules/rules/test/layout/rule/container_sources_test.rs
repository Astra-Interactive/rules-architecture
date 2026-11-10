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
