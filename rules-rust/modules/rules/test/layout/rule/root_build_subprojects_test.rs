use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::layout::rule::RootBuildSubprojects;
use crate::test::check::usecase::read_case_context;
use crate::workspace::util::parse_toml_file;

fn replace_root_manifest(context: &mut RuleContext, source: &str) {
    let manifest = parse_toml_file("Cargo.toml", source).expect("root manifest");
    context.workspace.root_manifest = manifest;
}

fn replace_member_manifest(context: &mut RuleContext, name: &str, source: &str) {
    for member in &mut context.workspace.members {
        if member.name == name {
            let parsed = parse_toml_file(&member.manifest.path, source).expect("member manifest");
            member.manifest = parsed;
        }
    }
}

fn replace_all_member_manifests(context: &mut RuleContext, source: &str) {
    for member in &mut context.workspace.members {
        let manifest = parse_toml_file(&member.manifest.path, source).expect("member manifest");
        member.manifest = manifest;
    }
}

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

#[test]
fn given_root_homepage_when_member_repeats_the_literal_then_inheritance_is_required() {
    let mut context = read_case_context("RootBuildSubprojects", "pass-inherited-settings");
    replace_root_manifest(
        &mut context,
        "[workspace]\n[workspace.package]\nhomepage = \"https://example.invalid/project\"\n",
    );
    replace_member_manifest(
        &mut context,
        "sample",
        "[package]\nname = \"sample\"\nhomepage = \"https://example.invalid/project\"\n",
    );
    replace_member_manifest(
        &mut context,
        "leaf",
        "[package]\nname = \"leaf\"\nhomepage.workspace = true\n",
    );
    let findings = RootBuildSubprojects.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.rule_id.0, "RootBuildSubprojects");
    assert_eq!(findings[0].finding.subject, "sample package.homepage");
    assert_eq!(findings[0].finding.location.line, Some(3));
}

#[test]
fn given_undeclared_root_dependency_when_two_members_repeat_the_requirement_then_the_second_is_reported()
 {
    let mut context = read_case_context("RootBuildSubprojects", "pass-inherited-settings");
    replace_root_manifest(&mut context, "[workspace]\n");
    replace_all_member_manifests(
        &mut context,
        "[package]\nname = \"member\"\n[dependencies]\nserde = \"1\"\n",
    );
    let second = context.workspace.members[1].name.clone();
    let findings = RootBuildSubprojects.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.rule_id.0, "RootBuildSubprojects");
    assert_eq!(
        findings[0].finding.subject,
        format!("{second} dependencies.serde")
    );
    assert_eq!(findings[0].finding.location.line, Some(4));
}

#[test]
fn given_string_and_version_only_table_when_members_repeat_the_requirement_then_the_second_is_reported()
 {
    let mut context = read_case_context("RootBuildSubprojects", "pass-inherited-settings");
    replace_root_manifest(&mut context, "[workspace]\n");
    let first = context.workspace.members[0].name.clone();
    let second = context.workspace.members[1].name.clone();
    replace_member_manifest(
        &mut context,
        &first,
        "[package]\nname = \"first\"\n[dependencies]\nfoo = \"1\"\nbar = \"1\"\nbaz = \"1\"\n",
    );
    replace_member_manifest(
        &mut context,
        &second,
        "[package]\nname = \"second\"\n[dependencies]\nfoo = { version = \"1\" }\nbar = { version = \"1\", features = [\"a\"] }\nbaz = { version = \"2\" }\n",
    );
    let findings = RootBuildSubprojects.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].finding.rule_id.0, "RootBuildSubprojects");
    assert_eq!(
        findings[0].finding.subject,
        format!("{second} dependencies.foo")
    );
    assert_eq!(findings[0].finding.location.line, Some(4));
}
