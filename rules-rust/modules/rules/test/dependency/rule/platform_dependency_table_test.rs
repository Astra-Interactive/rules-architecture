use crate::check::api::Rule;
use crate::dependency::rule::PlatformDependencyTable;
use crate::test::module::util::read_workspace_shape_context;

fn check_consumer(source: &str) -> Vec<String> {
    let context = read_workspace_shape_context(&[
        ("case_provider", "", "pub struct Thing;"),
        (
            "case_consumer",
            "case_provider={path=\"../case_provider\"}",
            source,
        ),
    ]);
    PlatformDependencyTable
        .check(&context)
        .iter()
        .map(|finding| {
            format!(
                "{}:{} {}",
                finding.finding.location.path,
                finding.finding.location.line.unwrap_or(0),
                finding.finding.subject
            )
        })
        .collect()
}

#[test]
fn given_dependency_used_only_under_a_target_predicate_when_checked_then_the_dependency_is_reported()
 {
    let findings = check_consumer(
        "#[cfg(target_os = \"linux\")]\nmod linux {\n    use case_provider::Thing;\n    pub(crate) fn read() -> Thing { Thing }\n}",
    );
    assert_eq!(
        findings,
        ["case_consumer/Cargo.toml:5 case_consumer -> case_provider"]
    );
}

#[test]
fn given_dependency_also_used_without_a_target_predicate_when_checked_then_nothing_is_reported() {
    let findings = check_consumer(
        "use case_provider::Thing;\n#[cfg(target_os = \"linux\")]\nmod linux {\n    use case_provider::Thing;\n    pub(crate) fn read() -> Thing { Thing }\n}\npub fn build() -> Thing { Thing }",
    );
    assert!(findings.is_empty(), "{findings:?}");
    let findings = check_consumer(
        "#[cfg(test)]\nmod test {\n    use case_provider::Thing;\n    fn read() -> Thing { Thing }\n}",
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn given_dependency_used_by_macro_or_attribute_path_without_a_target_predicate_when_checked_then_it_is_not_reported()
 {
    let linux_use = "#[cfg(target_os = \"linux\")]\nmod linux {\n    use case_provider::Thing;\n    pub(crate) fn read() -> Thing { Thing }\n}\n";
    let findings = check_consumer(&format!(
        "{linux_use}pub fn build() {{\n    case_provider::note!();\n}}"
    ));
    assert!(findings.is_empty(), "{findings:?}");
    let findings = check_consumer(&format!(
        "{linux_use}#[case_provider::mark]\npub fn build() {{}}"
    ));
    assert!(findings.is_empty(), "{findings:?}");
    let findings = check_consumer(
        "#[cfg(target_os = \"linux\")]\nmod linux {\n    use case_provider::Thing;\n    pub(crate) fn read() -> Thing {\n        case_provider::note!();\n        Thing\n    }\n}",
    );
    assert_eq!(
        findings,
        ["case_consumer/Cargo.toml:5 case_consumer -> case_provider"]
    );
}

#[test]
fn given_dependency_used_under_any_or_not_predicate_when_checked_then_only_all_with_a_target_atom_is_reported()
 {
    let findings = check_consumer(
        "#[cfg(any(target_os = \"linux\", feature = \"x\"))]\nmod gated {\n    use case_provider::Thing;\n    pub(crate) fn read() -> Thing { Thing }\n}",
    );
    assert!(findings.is_empty(), "{findings:?}");
    let findings = check_consumer(
        "#[cfg(not(windows))]\nmod gated {\n    use case_provider::Thing;\n    pub(crate) fn read() -> Thing { Thing }\n}",
    );
    assert!(findings.is_empty(), "{findings:?}");
    let findings = check_consumer(
        "#[cfg(all(unix, feature = \"x\"))]\nmod gated {\n    use case_provider::Thing;\n    pub(crate) fn read() -> Thing { Thing }\n}",
    );
    assert_eq!(
        findings,
        ["case_consumer/Cargo.toml:5 case_consumer -> case_provider"]
    );
}
