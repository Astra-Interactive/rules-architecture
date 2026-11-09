use crate::check::api::Rule;
use crate::naming::rule::KindNameInPackage;
use crate::test::check::usecase::read_case_context;

#[test]
fn given_technology_before_model_when_checked_then_keeps_representation_names() {
    let mut context = read_case_context("KindNameInPackage", "pass-syntax");
    context.project_config.crates[0]
        .technology_words
        .push("sqlx".to_owned());
    let file = context.crates[0]
        .files
        .iter_mut()
        .find(|file| !file.module.path.is_empty())
        .expect("source file");
    file.module.path.insert(0, "sqlx".to_owned());
    file.syntax = syn::parse_file("struct SubjectModel;").expect("valid source");
    assert!(KindNameInPackage.check(&context).is_empty());
}

#[test]
fn given_domain_and_technology_modules_when_checked_then_distinguishes_model_suffixes() {
    crate::test::syntax::util::assert_syntax_findings(
        &KindNameInPackage,
        "KindNameInPackage",
        "struct SubjectModel; enum SubjectEntity {} trait RowsUtils {} struct RowsManager; union RowUtil { value: u8 } struct RowsHelpers; mod api { trait LanguageModel {} }",
        6,
    );
}

#[test]
fn given_forbidden_syntax_when_checked_then_reports_the_rule() {
    let context = read_case_context("KindNameInPackage", "fail-syntax");
    assert!(!KindNameInPackage.check(&context).is_empty());
}

#[test]
fn given_prescribed_syntax_when_checked_then_reports_nothing() {
    let context = read_case_context("KindNameInPackage", "pass-syntax");
    assert!(KindNameInPackage.check(&context).is_empty());
}
