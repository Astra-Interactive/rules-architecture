use crate::check::api::Rule;
use crate::layout::rule::RootSources;
use crate::test::check::usecase::read_case_context;

#[test]
fn given_virtual_root_when_it_holds_an_empty_src_directory_then_the_directory_is_reported() {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    assert!(RootSources.check(&context).is_empty());
    context.workspace.directories.push("src".to_owned());
    let findings = RootSources.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.rule_id.0, "RootSources");
    assert_eq!(findings[0].finding.subject, ".");
    assert_eq!(findings[0].finding.location.path, "src");
    assert_eq!(findings[0].finding.location.line, None);
}
