use crate::check::api::Rule;
use crate::naming::rule::KindNameInPackage;
use crate::test::check::usecase::read_case_context;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;
use crate::test::declaration::util::read_code_context;

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
fn given_service_word_inside_non_model_type_name_when_checked_then_reports_only_whole_word() {
    assert_code_findings(
        &KindNameInPackage,
        "KindNameInPackage",
        "mod api { trait ServiceClient {} }\nmod service { struct ServiceClient; }\nmod util { struct ServiceableClient; }",
        &[1],
    );
}

#[test]
fn given_plural_roleless_words_when_checked_then_reports_each_but_not_similar_words() {
    assert_code_findings(
        &KindNameInPackage,
        "KindNameInPackage",
        "mod util { struct InvoiceServices; struct UtilityBill; }",
        &[1],
    );
}

#[test]
fn given_type_aliases_with_forbidden_names_when_checked_then_reports_them_like_declared_types() {
    let mut context = read_code_context(
        "mod model {\npub struct Jail { pub name: String }\npub type JailModel = Jail;\npub type JailEntity = Jail;\npub type InvoiceIndex = std::collections::HashMap<u32, Jail>;\n}\nmod file {\npub type RowHelper = u32;\npub type RowUtil = u32;\npub type RowUtils = u32;\npub type RowManager = u32;\npub type JailEntity = u32;\n}\npub type UserService = u32;\nmod service { pub type BillingService = u32; }\ntrait Source { type Item; }\nstruct Reader; impl Source for Reader { type RowHelper = u8; }\nmod sqlx {\n    pub mod model {\n        pub type JailEntity = u32;\n    }\n}",
    );
    context.project_config.crates[0]
        .technology_words
        .push("sqlx".to_owned());
    let mut lines = KindNameInPackage
        .check(&context)
        .iter()
        .map(|finding| finding.finding.location.line)
        .collect::<Vec<_>>();
    lines.sort_unstable();
    assert_eq!(
        vec![
            Some(3),
            Some(4),
            Some(8),
            Some(9),
            Some(10),
            Some(11),
            Some(14)
        ],
        lines
    );
}

#[test]
fn given_service_type_in_a_crate_named_service_when_checked_then_the_crate_name_grants_no_exemption()
 {
    let mut context = read_code_context(
        "mod payment {\n    mod usecase { pub(crate) struct PaymentService; }\n}\nmod service { pub(crate) struct ReminderService; }",
    );
    context.crates[0].crate_name = "billing-service".to_owned();
    for file in &mut context.crates[0].files {
        file.module.crate_name = "billing-service".to_owned();
    }
    let lines = KindNameInPackage
        .check(&context)
        .iter()
        .map(|finding| finding.finding.location.line)
        .collect::<Vec<_>>();
    assert_eq!(vec![Some(2)], lines);
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
