use crate::check::api::Rule;
use crate::check::model::RuleContext;
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
    let subjects: Vec<&str> = findings
        .iter()
        .map(|finding| finding.finding.subject.as_str())
        .collect();
    assert_eq!(
        subjects,
        [
            "case_sample::util::compute_value",
            "case_sample::util::combine_values"
        ]
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

fn list_lines(context: &RuleContext) -> Vec<u32> {
    let mut lines: Vec<u32> = UtilFunctionAsExtension
        .check(context)
        .iter()
        .map(|finding| finding.finding.location.line.expect("finding line"))
        .collect();
    lines.sort_unstable();
    lines
}

fn list_sites(context: &RuleContext) -> Vec<String> {
    let mut sites: Vec<String> = UtilFunctionAsExtension
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
    sites.sort();
    sites
}

fn read_claim_context(extension: &str) -> RuleContext {
    read_shape_context(
        "mod model { pub(crate) use claim::Claim; pub(crate) use claim_chunk::ClaimChunk; pub(crate) use claim_error::ClaimError; pub(crate) use claim_id::ClaimId; mod claim; mod claim_chunk; mod claim_error; mod claim_id; } mod util { mod claim_ext; }",
        &[
            ("model/claim.rs", "pub struct Claim;"),
            ("model/claim_chunk.rs", "pub struct ClaimChunk;"),
            ("model/claim_error.rs", "pub enum ClaimError { Full }"),
            ("model/claim_id.rs", "pub struct ClaimId(pub u64);"),
            ("util/claim_ext.rs", extension),
        ],
    )
}

#[test]
fn given_free_function_returning_the_type_of_a_model_parameter_when_checked_then_reports_the_transition()
 {
    let context = read_shape_context(
        "mod model { pub struct Claim; pub struct ClaimChunk; pub enum ClaimError { Full } }\nmod util {\nuse crate::model::{Claim, ClaimChunk, ClaimError};\nfn add_chunk(claim: Claim, chunk: ClaimChunk) -> Claim { claim }\nfn try_add_chunk(claim: &Claim, chunk: ClaimChunk) -> Result<Claim, ClaimError> { Err(ClaimError::Full) }\nfn find_chunk(claim: Claim, chunk: ClaimChunk) -> Option<ClaimChunk> { None }\nfn is_covered(claim: &Claim, chunk: &ClaimChunk) -> bool { true }\n}",
        &[],
    );
    assert_eq!(vec![4, 5, 6], list_lines(&context));
    let messages: Vec<String> = UtilFunctionAsExtension
        .check(&context)
        .iter()
        .map(|finding| finding.finding.message.clone())
        .collect();
    assert!(
        messages
            .iter()
            .all(|message| message
                .contains("a transition is an inherent method of the type it returns")),
        "{messages:?}"
    );
}

#[test]
fn given_free_function_with_one_model_parameter_among_several_when_checked_then_reports_it() {
    let context = read_shape_context(
        "mod model { pub struct Invoice; pub struct InvoiceId(pub u64); }\nmod util {\nuse crate::model::{Invoice, InvoiceId};\nuse std::path::Path;\nfn is_long(invoice: &Invoice, limit: u64) -> bool { true }\nfn is_owned_by(invoice: &Invoice, owner: InvoiceId) -> bool { true }\nfn describe(invoice: &Invoice, root: &Path, names: Vec<String>) -> String { String::new() }\nfn is_known(id: InvoiceId) -> bool { true }\nfn is_blank(text: &str, limit: u64) -> bool { true }\n}",
        &[],
    );
    assert_eq!(vec![5, 6, 7], list_lines(&context));
}

#[test]
fn given_method_of_an_out_of_file_impl_reading_another_model_when_checked_then_reports_it() {
    let context = read_claim_context(
        "use crate::model::{Claim, ClaimChunk};\nimpl Claim {\nfn covers(&self, chunk: &ClaimChunk) -> bool { true }\nfn is_empty(&self, limit: u32) -> bool { true }\nfn overlaps_chunk(&self, other: &Claim, chunk: &ClaimChunk) -> bool { true }\n}",
    );
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
        [
            "modules/sample/src/util/claim_ext.rs:3 case_sample::util::covers",
            "modules/sample/src/util/claim_ext.rs:5 case_sample::util::overlaps_chunk"
        ]
    );
}

#[test]
fn given_create_and_parse_functions_when_checked_then_they_are_exempt_and_other_names_report() {
    let context = read_shape_context(
        "mod model { pub struct Invoice; pub struct InvoiceDraft; pub enum InvoiceError { Empty } }\nmod util {\nuse crate::model::{Invoice, InvoiceDraft, InvoiceError};\nfn create_invoice(draft: InvoiceDraft, base: Invoice) -> Result<Invoice, InvoiceError> { Ok(base) }\nfn parse_invoice(draft: InvoiceDraft) -> Result<Invoice, InvoiceError> { Ok(Invoice) }\nfn parse_invoice_over(draft: InvoiceDraft, base: Invoice) -> Result<Invoice, InvoiceError> { Ok(base) }\nfn build_invoice(draft: InvoiceDraft) -> Result<Invoice, InvoiceError> { Ok(Invoice) }\n}",
        &[],
    );
    assert_eq!(vec![7], list_lines(&context));
}

#[test]
fn given_free_function_over_two_models_returning_neither_when_checked_then_it_is_not_reported() {
    let context = read_shape_context(
        "mod model { pub struct JailInmate; pub struct Money; }\nmod util {\nuse crate::model::{JailInmate, Money};\nfn is_bail_covered(inmate: &JailInmate, deposit: &Money) -> bool { true }\nfn compare_terms(first: &JailInmate, second: &Money) -> std::cmp::Ordering { std::cmp::Ordering::Equal }\nfn find_bail(inmate: &JailInmate, deposit: &Money) -> Option<u64> { None }\n}",
        &[],
    );
    assert_eq!(Vec::<u32>::new(), list_lines(&context));
}

#[test]
fn given_method_returning_its_own_type_with_other_model_parameters_when_checked_then_it_is_not_reported()
 {
    let context = read_claim_context(
        "use crate::model::{Claim, ClaimChunk, ClaimError};\nimpl Claim {\nfn add_chunk(self, chunk: ClaimChunk) -> Claim { self }\nfn merge_chunk(self, chunk: ClaimChunk) -> Self { self }\nfn try_add_chunk(self, chunk: ClaimChunk) -> Result<Self, ClaimError> { Ok(self) }\nfn find_chunk(self, chunk: ClaimChunk) -> Option<Claim> { None }\n}",
    );
    assert_eq!(Vec::<u32>::new(), list_lines(&context));
}

#[test]
fn given_method_whose_other_parameters_are_not_models_when_checked_then_it_is_not_reported() {
    let context = read_claim_context(
        "use crate::model::{Claim, ClaimId};\nuse std::path::Path;\nimpl Claim {\nfn is_owned_by(&self, owner: ClaimId) -> bool { true }\nfn is_long(&self, limit: u64) -> bool { true }\nfn describe(&self, root: &Path, names: Vec<String>) -> String { String::new() }\n}",
    );
    assert_eq!(Vec::<u32>::new(), list_lines(&context));
}

#[test]
fn given_signature_form_cases_when_checked_then_exact_wrong_sites_report_and_correct_forms_pass() {
    let context = read_case_context("UtilFunctionAsExtension", "fail-signature-form");
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
        [
            "modules/sample/src/util/claim_ext.rs:3 case_sample::util::covers",
            "modules/sample/src/util/claim_rules.rs:3 case_sample::util::add_chunk",
            "modules/sample/src/util/claim_rules.rs:4 case_sample::util::is_long",
            "modules/sample/src/util/claim_rules.rs:5 case_sample::util::exceeds"
        ]
    );
    let context = read_case_context("UtilFunctionAsExtension", "pass-signature-form");
    assert!(UtilFunctionAsExtension.check(&context).is_empty());
}

#[test]
fn given_method_whose_model_parameters_are_all_of_its_own_type_when_checked_then_it_is_not_reported()
 {
    let context = read_claim_context(
        "use crate::model::{Claim, ClaimId};\nuse std::sync::Arc;\nimpl Claim {\nfn overlaps(&self, other: &Claim) -> bool { true }\nfn is_between(&self, low: &Claim, high: Arc<Claim>) -> bool { true }\nfn exceeds(&self, limit: Claim, owner: ClaimId, minimum: u64) -> bool { true }\nfn matches(&self, other: &Self) -> bool { true }\n}",
    );
    assert_eq!(Vec::<u32>::new(), list_lines(&context));
}

#[test]
fn given_method_with_a_parameter_of_another_model_type_when_checked_then_it_is_reported_unless_it_returns_its_own_type()
 {
    let context = read_claim_context(
        "use crate::model::{Claim, ClaimChunk};\nimpl Claim {\nfn covers(&self, chunk: &ClaimChunk) -> bool { true }\nfn overlaps_chunk(&self, other: &Claim, chunk: &ClaimChunk) -> bool { true }\nfn compare_chunk(&self, chunk: ClaimChunk) -> Option<ClaimChunk> { None }\nfn with_chunk(self, other: &Claim, chunk: ClaimChunk) -> Claim { self }\n}",
    );
    assert_eq!(vec![3, 4, 5], list_lines(&context));
}

#[test]
fn given_free_function_whose_model_parameters_are_all_of_one_type_when_checked_then_it_is_a_method_of_that_type()
 {
    let context = read_shape_context(
        "mod model { pub struct Money; pub struct Rate; }\nmod util {\nuse crate::model::{Money, Rate};\nuse std::sync::Arc;\nfn exceeds(amount: Money, limit: Money) -> bool { true }\nfn is_within(amount: &Money, low: &Money, high: Arc<Money>) -> bool { true }\nfn is_between(amount: &Money, low: &Money, tolerance: u64) -> bool { true }\nfn exceeds_rate(amount: &Money, limit: &Money, rate: &Rate) -> bool { true }\n}",
        &[],
    );
    assert_eq!(vec![5, 6, 7], list_lines(&context));
}

#[test]
fn given_free_function_over_three_model_types_returning_none_of_them_when_checked_then_it_is_not_reported()
 {
    let context = read_shape_context(
        "mod model { pub struct JailInmate; pub struct Money; pub struct Cell; }\nmod util {\nuse crate::model::{Cell, JailInmate, Money};\nfn is_bail_covered(inmate: &JailInmate, deposit: &Money, cell: &Cell, other: &Money) -> bool { true }\n}",
        &[],
    );
    assert_eq!(Vec::<u32>::new(), list_lines(&context));
}

#[test]
fn given_model_type_declared_under_exclusive_cfgs_when_checked_then_the_variants_count_as_one_type()
{
    let context = read_shape_context(
        "mod model {\n#[cfg(unix)] pub struct Invoice;\n#[cfg(not(unix))] pub struct Invoice;\npub struct Chunk;\n}\nmod util {\nuse crate::model::{Chunk, Invoice};\nfn is_paid(invoice: &Invoice) -> bool { true }\nfn exceeds(first: &Invoice, second: &Invoice) -> bool { true }\nfn add_chunk(invoice: Invoice, chunk: Chunk) -> Invoice { invoice }\nfn is_covered(invoice: &Invoice, chunk: &Chunk) -> bool { true }\n}",
        &[],
    );
    assert_eq!(vec![8, 9, 10], list_lines(&context));
}

#[test]
fn given_model_type_visible_only_under_a_feature_when_checked_then_a_function_without_the_feature_does_not_see_it()
 {
    let context = read_shape_context(
        "mod model { #[cfg(feature = \"fast\")] pub struct Invoice; }\nmod util {\nuse crate::model::Invoice;\n#[cfg(feature = \"fast\")]\nfn is_valid(invoice: &Invoice) -> bool { true }\n#[cfg(not(feature = \"fast\"))]\nfn is_paid(invoice: &Invoice) -> bool { true }\n}",
        &[],
    );
    let subjects: Vec<String> = UtilFunctionAsExtension
        .check(&context)
        .into_iter()
        .map(|finding| finding.finding.subject)
        .collect();
    assert_eq!(subjects, ["case_sample::util::is_valid"]);
}

#[test]
fn given_helpers_in_the_util_blocks_of_the_test_and_fixture_trees_when_checked_then_only_the_src_tree_reports()
 {
    let rule_file = "use crate::model::Invoice;\nfn is_paid(invoice: &Invoice) -> bool { true }\nfn assert_same(actual: &Invoice, expected: &Invoice) {}";
    let extension_file = "use crate::model::{Chunk, Invoice};\nimpl Invoice {\nfn covers(&self, chunk: &Chunk) -> bool { true }\n}";
    let context = read_shape_context(
        "mod model { pub struct Invoice; pub struct Chunk; }\nmod util { mod invoice_rule; mod invoice_ext; }\n#[cfg(any(test, feature = \"fake\"))]\nmod fixture { mod util { mod invoice_rule; mod invoice_ext; } }\n#[cfg(test)]\nmod test { mod util { mod invoice_rule_test; mod invoice_ext_test; } }",
        &[
            ("util/invoice_rule.rs", rule_file),
            ("util/invoice_ext.rs", extension_file),
            ("fixture/util/invoice_rule.rs", rule_file),
            ("fixture/util/invoice_ext.rs", extension_file),
            ("test/util/invoice_rule_test.rs", rule_file),
            ("test/util/invoice_ext_test.rs", extension_file),
        ],
    );
    assert_eq!(
        list_sites(&context),
        [
            "modules/sample/src/util/invoice_ext.rs:3 case_sample::util::covers",
            "modules/sample/src/util/invoice_rule.rs:2 case_sample::util::is_paid",
            "modules/sample/src/util/invoice_rule.rs:3 case_sample::util::assert_same",
        ]
    );
}
