use crate::check::api::Rule;
use crate::module::rule::UtilShape;
use crate::test::check::usecase::read_case_context;
use crate::test::module::util::read_shape_context;

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
fn given_file_constant_in_util_function_file_when_checked_then_only_static_and_type_report() {
    let context = read_shape_context(
        "mod util { mod money_parse; mod limit_state; mod row; }",
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
        ["case_sample::util::LIMIT", "case_sample::util::Row"]
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
