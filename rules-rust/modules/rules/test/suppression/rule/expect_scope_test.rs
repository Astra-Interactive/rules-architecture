use crate::check::api::Rule;
use crate::check::model::RuleFinding;
use crate::config::model::Role;
use crate::suppression::rule::ExpectScope;
use crate::test::check::usecase::read_case_context;

fn check_source_in_role(source: &str, role: Role) -> Vec<RuleFinding> {
    let mut context = read_case_context("ExpectScope", "pass-syntax");
    let file = context.crates[0]
        .files
        .iter_mut()
        .find(|file| !file.module.path.is_empty())
        .expect("fixture has a source file");
    file.syntax = syn::parse_file(source).expect("test input parses");
    let crate_name = file.module.crate_name.clone();
    context
        .project_config
        .crates
        .iter_mut()
        .find(|config| config.name == crate_name)
        .expect("fixture crate has a configuration")
        .role = Some(role);
    ExpectScope.check(&context)
}

#[test]
fn given_unrelated_or_empty_attributes_when_checked_then_reports_no_lint_expectation() {
    crate::test::syntax::util::assert_syntax_findings(
        &ExpectScope,
        "ExpectScope",
        "#[other] #[other(=>)] #[other(value)] #[expect(reason = \"fixture\")] fn run() {}",
        0,
    );
}

#[test]
fn given_inner_module_or_nonprescribed_expectations_when_checked_then_reports_scope() {
    crate::test::syntax::util::assert_syntax_findings(
        &ExpectScope,
        "ExpectScope",
        "#![allow(dead_code)] #![expect(clippy::unwrap_used, reason = \"fixture\")] #[expect(clippy::unwrap_used, reason = \"fixture\")] mod nested { #[expect(clippy::expect_used, reason = \"fixture\")] fn valid() {} } #[cfg_attr(feature = \"case\", expect(dead_code))] fn invalid() {}",
        4,
    );
}

#[test]
fn given_expression_lint_expected_on_an_impl_or_trait_when_checked_then_reports_the_container() {
    let source = "struct ReportCommand;\n#[expect(clippy::print_stdout, reason = \"the report is the product\")]\nimpl ReportCommand {\n    fn run(&self) { println!(\"report\"); }\n    #[expect(clippy::print_stdout, reason = \"the summary is the product\")]\n    fn summary(&self) { println!(\"summary\"); }\n}\n#[expect(clippy::unwrap_used, reason = \"fixture\")]\ntrait Reader { fn read(&self) -> u8 { Some(1).unwrap() } }\n#[expect(clippy::disallowed_types, reason = \"the library demands a std lock\")]\nimpl Holder<std::sync::Mutex<u8>> {}";
    let mut lines: Vec<_> = check_source_in_role(source, Role::EntryPoint)
        .iter()
        .map(|finding| finding.finding.location.line)
        .collect();
    lines.sort_unstable();
    assert_eq!(lines, [Some(2), Some(8)]);
}

#[test]
fn given_prescribed_lints_and_architecture_expectations_when_checked_then_keeps_them() {
    let source = "#[cfg_attr(klibs_rules, expect(UnknownRule, reason = \"fixture\"))] mod nested {} #[expect(clippy::unwrap_used, clippy::expect_used, clippy::let_underscore_must_use, clippy::disallowed_methods, clippy::disallowed_types, clippy::print_stdout, reason = \"fixture\")] fn valid() {} #[cfg_attr(feature = \"case\", expect(clippy::expect_used, reason = \"fixture\"))] fn conditional() {}";
    let findings = check_source_in_role(source, Role::EntryPoint);
    assert_eq!(0, findings.len(), "findings: {findings:?}");
}

#[test]
fn given_forbidden_syntax_when_checked_then_reports_the_rule() {
    let context = read_case_context("ExpectScope", "fail-syntax");
    assert!(!ExpectScope.check(&context).is_empty());
}

#[test]
fn given_prescribed_syntax_when_checked_then_reports_nothing() {
    let context = read_case_context("ExpectScope", "pass-syntax");
    assert!(ExpectScope.check(&context).is_empty());
}

#[test]
fn given_allow_under_conditional_cfg_attr_when_checked_then_reports_each_attribute() {
    let source = "#[cfg_attr(any(), allow(dead_code))]\nfn first() {}\n#[cfg_attr(feature = \"x\", allow(dead_code))]\nfn second() {}\n#[cfg_attr(feature = \"x\", expect(clippy::unwrap_used, reason = \"fixture\"))]\nfn third() {}\n";
    let findings = check_source_in_role(source, Role::Context);
    let lines: Vec<Option<u32>> = findings
        .iter()
        .map(|finding| {
            assert_eq!("ExpectScope", finding.finding.rule_id.0);
            finding.finding.location.line
        })
        .collect();
    assert_eq!(vec![Some(1), Some(3)], lines, "findings: {findings:?}");
}

#[test]
fn given_print_stdout_expectation_when_checked_then_only_entry_and_tooling_crates_keep_it() {
    let source = "#[expect(clippy::print_stdout, reason = \"report\")]\nfn show(total: u64) {\n    println!(\"{total}\");\n}\n";
    for role in [Role::EntryPoint, Role::Tooling] {
        let findings = check_source_in_role(source, role);
        assert!(
            findings.is_empty(),
            "role: {role:?}\nfindings: {findings:?}"
        );
    }
    let findings = check_source_in_role(source, Role::Context);
    assert_eq!(1, findings.len(), "findings: {findings:?}");
    assert_eq!("ExpectScope", findings[0].finding.rule_id.0);
    assert_eq!(Some(1), findings[0].finding.location.line);
}
