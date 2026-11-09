use crate::check::api::Rule;
use crate::module::rule::DiPlacement;
use crate::test::check::usecase::read_case_context;
use crate::test::module::util::read_shape_context;

#[test]
fn given_contracts_only_crate_when_checked_then_di_block_is_reported_but_implementation_crate_passes()
 {
    let context = read_shape_context(
        "mod api { trait Contract {} } mod model { struct Value; } mod di {}",
        &[],
    );
    let findings = DiPlacement.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.subject, "case_sample::di");
    let context = read_shape_context(
        "mod api { trait Contract {} } mod file { struct Value; } mod di {}",
        &[],
    );
    assert!(DiPlacement.check(&context).is_empty());
}

#[test]
fn given_declaration_shapes_when_checked_then_exact_wrong_sites_report_and_correct_shapes_pass() {
    let context = read_case_context("DiPlacement", "fail-declaration-shape");
    let mut actual: Vec<String> = DiPlacement
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
        ["modules/sample/src/util/invoice_module.rs:1 case_sample::util::InvoiceModule"]
    );
    let context = read_case_context("DiPlacement", "pass-declaration-shape");
    assert!(DiPlacement.check(&context).is_empty());
}
