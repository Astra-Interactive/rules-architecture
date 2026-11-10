use crate::check::api::Rule;
use crate::naming::rule::KindNameInPackage;
use crate::test::check::usecase::read_case_context;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_persistent_code_fixtures_when_checked_then_match_exact_anchors() {
    assert_code_fixture(&KindNameInPackage, "KindNameInPackage", "bad");
    assert_code_fixture(&KindNameInPackage, "KindNameInPackage", "good");
}

#[test]
fn given_service_and_api_suffixes_when_checked_then_reports_only_misplaced_type_names() {
    assert_code_findings(
        &KindNameInPackage,
        "KindNameInPackage",
        "mod model { struct UserService; }\nmod api { trait UserApi {} trait UserDao {} trait UserKrate {} trait UserStorage {} }",
        &[1, 2, 2, 2, 2],
    );
    assert_code_findings(
        &KindNameInPackage,
        "KindNameInPackage",
        "mod service { struct UserService; }\nmod api { trait UserRepository {} }\nmod file { struct UserStorage; }",
        &[],
    );
}

#[test]
fn given_service_word_inside_model_type_name_when_checked_then_reports_it() {
    assert_code_findings(
        &KindNameInPackage,
        "KindNameInPackage",
        "mod model { struct InvoiceServiceState { first: u32, second: u32 } }\nmod service { struct InvoiceServiceState; }",
        &[1],
    );
}

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
