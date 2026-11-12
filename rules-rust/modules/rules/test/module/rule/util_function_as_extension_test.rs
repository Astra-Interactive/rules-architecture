use crate::check::api::Rule;
use crate::module::rule::UtilFunctionAsExtension;
use crate::test::check::usecase::read_case_context;
use crate::test::module::util::read_shape_context;
use crate::test::module::util::read_workspace_shape_context;

#[test]
fn given_reexport_alias_and_exclusive_cfg_type_when_checked_then_only_compatible_model_parameters_report()
 {
    let context = read_shape_context(
        "mod model { #[cfg(unix)] pub struct Invoice; } mod bridge { pub use crate::model::Invoice as Alias; } mod util { use crate::bridge::Alias as Local; #[cfg(unix)] fn compute_value(value: &mut Local) {} #[cfg(not(unix))] fn read_value(value: &Local) {} fn combine_values(value: &Local, tax: u32) {} }",
        &[],
    );
    let findings = UtilFunctionAsExtension.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings[0].finding.subject,
        "case_sample::util::compute_value"
    );
}

#[test]
fn given_declaration_shapes_when_checked_then_exact_wrong_sites_report_and_correct_shapes_pass() {
    let context = read_case_context("UtilFunctionAsExtension", "fail-declaration-shape");
    let mut actual: Vec<String> = UtilFunctionAsExtension
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
        ["modules/sample/src/util/invoice_total.rs:2 case_sample::util::compute_total"]
    );
    let context = read_case_context("UtilFunctionAsExtension", "pass-declaration-shape");
    assert!(UtilFunctionAsExtension.check(&context).is_empty());
}

#[test]
fn given_util_function_on_foreign_crate_model_when_checked_then_reports_extension_gap() {
    let context = read_workspace_shape_context(&[
        (
            "shop_billing_api",
            "",
            "pub mod model { pub struct Invoice { pub total: u64, pub paid_at: Option<std::time::SystemTime> } }",
        ),
        (
            "shop_billing_impl",
            "shop_billing_api={path=\"../shop_billing_api\"}",
            "mod util {\n\
             use shop_billing_api::model::Invoice;\n\
             fn is_invoice_paid(invoice: &Invoice) -> bool { invoice.paid_at.is_some() }\n\
             fn is_blank(text: &str) -> bool { text.is_empty() }\n\
             }",
        ),
    ]);
    let lines: Vec<Option<u32>> = UtilFunctionAsExtension
        .check(&context)
        .iter()
        .map(|finding| finding.finding.location.line)
        .collect();
    assert_eq!(vec![Some(3)], lines);
}

#[test]
fn given_util_function_on_smart_pointer_of_model_when_checked_then_reports_extension_gap() {
    let context = read_shape_context(
        "mod model { pub struct Invoice; }\nmod util {\nuse crate::model::Invoice;\nuse std::sync::Arc;\nfn compute_total(invoice: Arc<Invoice>) -> u64 { 0 }\nfn compute_tax(invoice: &Box<Invoice>) -> u64 { 0 }\nfn count_invoices(invoices: Vec<Invoice>) -> usize { 0 }\nfn is_settled(invoice: Option<&Invoice>) -> bool { false }\n}",
        &[],
    );
    let lines: Vec<Option<u32>> = UtilFunctionAsExtension
        .check(&context)
        .iter()
        .map(|finding| finding.finding.location.line)
        .collect();
    assert_eq!(vec![Some(5), Some(6)], lines);
}

#[test]
fn given_create_function_returning_model_result_when_checked_then_only_other_functions_report() {
    let context = read_shape_context(
        "mod model { pub struct Invoice; pub struct InvoiceDraft; pub enum InvoiceError { Empty } }\nmod util {\nuse crate::model::{Invoice, InvoiceDraft, InvoiceError};\nfn create_invoice(draft: InvoiceDraft) -> Result<Invoice, InvoiceError> { Ok(Invoice) }\nfn create_total(invoice: &Invoice) -> u64 { 0 }\nfn build_invoice(draft: InvoiceDraft) -> Result<Invoice, InvoiceError> { Ok(Invoice) }\n}",
        &[],
    );
    let lines: Vec<Option<u32>> = UtilFunctionAsExtension
        .check(&context)
        .iter()
        .map(|finding| finding.finding.location.line)
        .collect();
    assert_eq!(vec![Some(5), Some(6)], lines);
}
