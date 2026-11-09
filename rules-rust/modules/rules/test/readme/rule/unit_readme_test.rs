use crate::check::api::Rule;
use crate::readme::rule::UnitReadme;
use crate::test::check::usecase::read_case_context;
use crate::workspace::util::parse_toml_file;

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
