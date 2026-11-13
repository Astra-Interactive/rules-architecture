use crate::check::api::Rule;
use crate::module::rule::UnsafePlacement;
use crate::test::declaration::util::{
    assert_code_findings, assert_code_fixture, read_code_context,
};

#[test]
fn given_persistent_placement_cases_when_checked_then_match_exact_node_anchors() {
    assert_code_fixture(&UnsafePlacement, "UnsafePlacement", "bad");
    assert_code_fixture(&UnsafePlacement, "UnsafePlacement", "good");
}

#[test]
fn given_unsafe_extern_block_in_util_kind_when_checked_then_the_block_is_reported() {
    assert_code_findings(
        &UnsafePlacement,
        "UnsafePlacement",
        "mod util {\n    unsafe extern \"C\" {\n        safe fn abs(input: i32) -> i32;\n    }\n    fn load() { unsafe {} }\n}",
        &[2, 5],
    );
}

#[test]
fn given_unsafe_attributes_outside_a_technology_module_when_checked_then_each_attribute_is_reported()
 {
    assert_code_findings(
        &UnsafePlacement,
        "UnsafePlacement",
        "mod util {\n    #[unsafe(no_mangle)]\n    fn exported() {}\n    #[unsafe(export_name = \"shop_run\")]\n    fn run() {}\n    #[inline]\n    fn plain() {}\n}\nmod file { #[unsafe(no_mangle)] fn exported() {} }",
        &[2, 4],
    );
}

#[test]
fn given_unsafe_in_platform_crate_concept_when_checked_then_only_the_util_kind_is_reported() {
    let mut context = read_code_context(
        "mod menu {\n    fn open() { unsafe {} }\n}\nmod util {\n    fn load() { unsafe {} }\n}",
    );
    context.project_config.crates[0].role = Some(crate::config::model::Role::Platform);
    let findings = UnsafePlacement.check(&context);
    let mut lines: Vec<_> = findings
        .iter()
        .map(|finding| {
            assert_eq!("UnsafePlacement", finding.finding.rule_id.0);
            finding.finding.location.line.expect("finding line")
        })
        .collect();
    lines.sort_unstable();
    assert_eq!(vec![5], lines, "{findings:?}");
}
