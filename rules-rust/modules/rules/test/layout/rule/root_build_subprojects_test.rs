use crate::check::api::Rule;
use crate::layout::rule::RootBuildSubprojects;
use crate::test::check::usecase::read_case_context;

#[test]
fn given_shared_settings_when_checked_then_literal_package_and_all_dependency_kinds_require_inheritance()
 {
    let context = read_case_context("RootBuildSubprojects", "fail-literal-settings");
    let mut subjects: Vec<String> = RootBuildSubprojects
        .check(&context)
        .into_iter()
        .map(|finding| finding.finding.subject)
        .collect();
    subjects.sort();
    assert_eq!(
        subjects,
        [
            "leaf replace",
            "sample build-dependencies.leaf",
            "sample dependencies.leaf",
            "sample dev-dependencies.leaf",
            "sample package.authors",
            "sample package.edition",
            "sample package.license",
            "sample package.repository",
            "sample package.rust-version",
            "sample patch",
            "sample profile",
            "sample target.cfg(windows).dependencies.leaf",
        ]
    );
    let context = read_case_context("RootBuildSubprojects", "pass-inherited-settings");
    assert!(RootBuildSubprojects.check(&context).is_empty());
}
