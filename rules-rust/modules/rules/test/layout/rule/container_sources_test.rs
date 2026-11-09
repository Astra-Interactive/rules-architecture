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
