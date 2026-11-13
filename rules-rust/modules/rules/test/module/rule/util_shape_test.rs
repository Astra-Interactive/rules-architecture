use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::config::model::Role;
use crate::module::rule::UtilShape;
use crate::test::check::usecase::read_case_context;
use crate::test::module::util::read_import_context;
use crate::test::module::util::read_shape_context;
use crate::test::module::util::read_workspace_shape_context;

#[test]
fn given_generated_util_data_when_checked_then_generated_file_is_exempt() {
    let context = read_shape_context(
        "mod util { mod values; }",
        &[(
            "util/values.rs",
            "// This file is @generated.\nstruct RuleValues;",
        )],
    );
    assert!(UtilShape.check(&context).is_empty());
}

#[test]
fn given_reexported_type_and_cfg_extensions_when_checked_then_only_out_of_file_non_extension_impl_reports()
 {
    let mut context = read_shape_context(
        "mod model { #[cfg(unix)] pub struct Invoice; pub use Invoice as Alias; } mod command { #[cfg(unix)] impl crate::model::Alias {} } mod util { #[cfg(unix)] impl crate::model::Alias {} }",
        &[],
    );
    assert!(UtilShape.check(&context).is_empty());
    for item in &mut context.indexes[0].items {
        if item.name == "Invoice" {
            item.file = "model/invoice.rs".to_owned();
        }
    }
    assert_eq!(UtilShape.check(&context).len(), 1);
    context.workspace.root_manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[workspace.metadata.klibs-rules]\nextension-kinds=['command']",
    )
    .expect("manifest");
    assert!(UtilShape.check(&context).is_empty());
}

#[test]
fn given_file_constant_in_util_function_file_when_checked_then_only_lone_data_and_type_report() {
    let context = read_shape_context(
        "mod util { mod money_parse; mod limit_state; mod row; mod lone; }",
        &[
            (
                "util/money_parse.rs",
                "const MAX_CENTS: i64 = 100;\npub(crate) fn parse_money() -> i64 { MAX_CENTS }",
            ),
            (
                "util/limit_state.rs",
                "static LIMIT: i64 = 100;\npub(crate) fn read_limit() -> i64 { LIMIT }",
            ),
            ("util/row.rs", "struct Row;"),
            ("util/lone.rs", "static LONE: i64 = 1;"),
        ],
    );
    let mut subjects: Vec<_> = UtilShape
        .check(&context)
        .iter()
        .map(|finding| finding.finding.subject.clone())
        .collect();
    subjects.sort();
    assert_eq!(
        subjects,
        ["case_sample::util::LONE", "case_sample::util::Row"]
    );
}

#[test]
fn given_declaration_shapes_when_checked_then_exact_wrong_sites_report_and_correct_shapes_pass() {
    let context = read_case_context("UtilShape", "fail-declaration-shape");
    let mut actual: Vec<String> = UtilShape
        .check(&context)
        .iter()
        .map(|finding| {
            format!(
                "{}:{} {}",
                finding.finding.location.path,
                finding.finding.location.line.expect("finding line"),
                finding.finding.subject
            )
        })
        .collect();
    actual.sort();
    assert_eq!(
        actual,
        ["modules/sample/src/util/values.rs:1 case_sample::util::RuleValues"]
    );
    let context = read_case_context("UtilShape", "pass-declaration-shape");
    assert!(UtilShape.check(&context).is_empty());
}

fn list_findings(context: &RuleContext) -> Vec<String> {
    let mut lines: Vec<String> = UtilShape
        .check(context)
        .iter()
        .map(|finding| {
            format!(
                "{}:{} {}",
                finding.finding.location.path,
                finding.finding.location.line.expect("finding line"),
                finding.finding.subject
            )
        })
        .collect();
    lines.sort();
    lines
}

fn list_lines(context: &RuleContext) -> Vec<u32> {
    let mut lines: Vec<u32> = UtilShape
        .check(context)
        .iter()
        .map(|finding| finding.finding.location.line.expect("finding line"))
        .collect();
    lines.sort_unstable();
    lines
}

#[test]
fn given_util_functions_with_project_trait_parameters_when_checked_then_each_form_reports() {
    let context = read_shape_context(
        "mod api {\npub trait Repository {}\n}\nmod model {\npub struct Invoice;\n}\nmod util {\nuse crate::api::Repository;\nuse std::sync::Arc;\nfn load_all(repository: &dyn Repository) {}\nfn store_all(repository: Arc<dyn Repository>) {}\nfn drop_all(repository: Box<dyn Repository>) {}\nfn scan_all(repository: impl Repository) {}\nfn walk_all<R: Repository>(repository: R) {}\nfn sweep_all<R>(repository: R) where R: Repository {}\n}",
        &[],
    );
    assert_eq!(list_lines(&context), [10, 11, 12, 13, 14, 15]);
}

#[test]
fn given_util_functions_with_nested_and_aliased_trait_parameters_when_checked_then_each_reports() {
    let context = read_shape_context(
        "mod api {\npub trait Repository {}\n}\nmod util {\nuse crate::api::Repository as Contract;\nuse crate::api::Repository;\nfn find_all(repository: Option<&dyn Repository>) {}\nfn load_all(repositories: Vec<Box<dyn Repository>>) {}\nfn scan_all<R: Repository>(repository: Option<&R>) {}\nfn walk_all<R: Clone + Repository>(repository: R) {}\nfn sweep_all<R: Clone>(repository: R) where R: Repository {}\nfn drop_all(repository: &dyn Contract) {}\nfn clone_all<R: Clone>(repository: R) {}\n}",
        &[],
    );
    assert_eq!(list_lines(&context), [7, 8, 9, 10, 11, 12]);
}

#[test]
fn given_util_functions_over_model_host_and_library_types_when_checked_then_none_reports() {
    let context = read_import_context(
        "mod api {\npub trait Repository {}\n}\nmod model {\npub struct Invoice;\n}\nmod usecase {\nuse crate::api::Repository;\nfn invoke(repository: &dyn Repository) {}\n}\nmod fake {\nuse crate::api::Repository;\nfn invoke(repository: &dyn Repository) {}\n}\nmod util {\nuse crate::model::Invoice;\nuse host_api::Player;\nuse std::sync::{Arc, Mutex};\nuse std::time::SystemTime;\nfn compute_total(invoice: &Invoice, rate: u32) -> u64 { 0 }\nfn kick_player(player: &Player) {}\nfn read_moment(moment: SystemTime) {}\nfn count_all(items: &[Invoice], cache: Arc<Mutex<u32>>) {}\nfn apply_step(step: impl Fn(u32) -> u32) {}\nfn run_step<F: Fn(u32) -> u32>(step: F) {}\nfn show_all<T>(value: T) where T: std::fmt::Debug {}\n}",
        Role::Context,
        &["host"],
        &[],
        "app",
    );
    assert_eq!(list_lines(&context), Vec::<u32>::new());
}

#[test]
fn given_util_method_of_an_out_of_file_impl_with_a_trait_parameter_when_checked_then_the_method_reports()
 {
    let context = read_shape_context(
        "mod api { pub trait Repository {} } mod model { pub(crate) use invoice::Invoice; mod invoice; } mod util { mod invoice_ext; }",
        &[
            ("model/invoice.rs", "pub struct Invoice;"),
            (
                "util/invoice_ext.rs",
                "use crate::api::Repository;\nuse crate::model::Invoice;\nimpl Invoice {\nfn settle(&self, repository: &dyn Repository) {}\nfn is_settled(&self, now: u64) -> bool { true }\n}",
            ),
        ],
    );
    assert_eq!(
        list_findings(&context),
        ["modules/sample/src/util/invoice_ext.rs:4 case_sample::util::settle"]
    );
}

#[test]
fn given_util_function_over_a_contract_of_another_crate_when_checked_then_reports_it() {
    let context = read_workspace_shape_context(&[
        (
            "shop_billing_api",
            "",
            "pub mod api { pub trait InvoiceRepository {} }",
        ),
        (
            "shop_billing_impl",
            "shop_billing_api={path=\"../shop_billing_api\"}",
            "mod util {\nuse shop_billing_api::api::InvoiceRepository;\nfn load_all(repository: &dyn InvoiceRepository) {}\nfn is_blank(text: &str) -> bool { text.is_empty() }\n}",
        ),
    ]);
    assert_eq!(list_lines(&context), [3]);
}

#[test]
fn given_contract_parameter_fixtures_when_checked_then_exact_wrong_functions_report_and_others_pass()
 {
    let context = read_case_context("UtilShape", "fail-contract-parameter");
    assert_eq!(
        list_findings(&context),
        [
            "modules/sample/src/util/invoice_ext.rs:5 case_sample::util::settle",
            "modules/sample/src/util/repository_scan.rs:10 case_sample::util::scan_all",
            "modules/sample/src/util/repository_scan.rs:12 case_sample::util::walk_all",
            "modules/sample/src/util/repository_scan.rs:14 case_sample::util::sweep_all",
            "modules/sample/src/util/repository_scan.rs:4 case_sample::util::load_all",
            "modules/sample/src/util/repository_scan.rs:6 case_sample::util::store_all",
            "modules/sample/src/util/repository_scan.rs:8 case_sample::util::drop_all",
        ]
    );
    let context = read_case_context("UtilShape", "pass-contract-parameter");
    assert!(list_findings(&context).is_empty());
}

#[test]
fn given_same_name_traits_under_exclusive_feature_cfgs_when_checked_then_only_the_function_that_sees_the_api_trait_reports()
 {
    let context = read_shape_context(
        "mod api {\n#[cfg(feature = \"fast\")]\npub trait Repository {}\n}\nmod legacy {\n#[cfg(not(feature = \"fast\"))]\npub trait Repository {}\n}\nmod util {\n#[cfg(feature = \"fast\")]\nuse crate::api::Repository;\n#[cfg(not(feature = \"fast\"))]\nuse crate::legacy::Repository;\n#[cfg(not(feature = \"fast\"))]\nfn load_legacy(repository: &dyn Repository) {}\n#[cfg(feature = \"fast\")]\nfn load_fast(repository: &dyn Repository) {}\n}",
        &[],
    );
    let subjects: Vec<String> = UtilShape
        .check(&context)
        .into_iter()
        .map(|finding| finding.finding.subject)
        .collect();
    assert_eq!(subjects, ["case_sample::util::load_fast"]);
}

#[test]
fn given_contract_parameters_in_the_util_blocks_of_the_test_and_fixture_trees_when_checked_then_only_the_src_tree_reports()
 {
    let function_file = "use crate::api::Repository;\nfn load_all(repository: &dyn Repository) {}";
    let extension_file = "use crate::api::Repository;\nuse crate::model::Invoice;\nimpl Invoice {\nfn settle(&self, repository: &dyn Repository) {}\n}";
    let context = read_shape_context(
        "mod api { pub trait Repository {} }\nmod model { pub struct Invoice; }\nmod util { mod repository_scan; mod invoice_ext; }\n#[cfg(any(test, feature = \"fake\"))]\nmod fixture { mod util { mod repository_scan; mod invoice_ext; } }\n#[cfg(test)]\nmod test { mod util { mod repository_scan_test; mod invoice_ext_test; } }",
        &[
            ("util/repository_scan.rs", function_file),
            ("util/invoice_ext.rs", extension_file),
            ("fixture/util/repository_scan.rs", function_file),
            ("fixture/util/invoice_ext.rs", extension_file),
            ("test/util/repository_scan_test.rs", function_file),
            ("test/util/invoice_ext_test.rs", extension_file),
        ],
    );
    assert_eq!(
        list_findings(&context),
        [
            "modules/sample/src/util/invoice_ext.rs:4 case_sample::util::settle",
            "modules/sample/src/util/repository_scan.rs:2 case_sample::util::load_all",
        ]
    );
}
