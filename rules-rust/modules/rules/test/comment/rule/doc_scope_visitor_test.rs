use crate::check::api::Rule;
use crate::check::model::Anchor;
use crate::comment::rule::DocCommentScope;
use crate::test::declaration::util::read_code_context;

#[test]
fn given_external_reason_and_additive_trait_contract_when_checked_then_does_not_require_links() {
    let context = read_code_context(
        "/// Windows may deliver a resume event before the device is available.\nfn resume() {}\ntrait Store { fn store(&self, value: u8); }\nstruct Buffer; impl Store for Buffer {\n/// Values are persisted after the next flush.\nfn store(&self, value: u8) {}\n}\n/// Resume.\nfn resume_again() {}",
    );
    assert!(DocCommentScope.check(&context).is_empty());
}

#[test]
fn given_copied_trait_contract_when_checked_then_rejects_only_the_unchanged_implementation_docs() {
    let context = read_code_context(
        "trait Store {\n/// Values are stored until the next flush.\nfn store(&self, value: u8);\n}\nstruct Buffer; impl Store for Buffer {\n/// Values are stored until the next flush.\nfn store(&self, value: u8) {}\n}\nstruct File; impl Store for File {\n/// Values are persisted on the next filesystem flush.\nfn store(&self, value: u8) {}\n}",
    );
    let lines: Vec<_> = DocCommentScope
        .check(&context)
        .iter()
        .map(|finding| finding.finding.location.line)
        .collect();
    assert_eq!(vec![Some(6)], lines);
}

#[test]
fn given_safe_index_forms_and_shadowed_receivers_when_published_then_does_not_invent_panics() {
    let mut context = read_code_context(
        "pub fn first(values: &[u8; 2]) -> u8 { values[0] }\npub fn full(values: &[u8]) -> &[u8] { &values[..] }\npub fn custom(values: &[u8]) -> u8 { let values = Custom; values[0] }\nstruct Custom;",
    );
    context.workspace.members[0].manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[package]\nname=\"case-sample\"\nversion=\"0.1.0\"",
    )
    .expect("published manifest");
    assert!(DocCommentScope.check(&context).is_empty());
}

#[test]
fn given_published_indexing_and_trait_defaults_when_checked_then_requires_panic_contracts() {
    let mut context = read_code_context(
        "pub fn element(values: &[u8], index: usize) -> u8 { values[index] }\ntrait Reader { fn read(&self, values: &[u8], index: usize) -> u8 { values[index] } }\n/// # Panics\n/// Panics when index is outside values.\npub fn documented(values: &[u8], index: usize) -> u8 { values[index] }\nstruct Custom; pub fn custom(values: Custom, index: usize) -> u8 { values[index] }",
    );
    context.workspace.members[0].manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[package]\nname=\"case-sample\"\nversion=\"0.1.0\"",
    )
    .expect("published manifest");
    let lines: Vec<_> = DocCommentScope
        .check(&context)
        .iter()
        .map(|finding| finding.finding.location.line)
        .collect();
    assert_eq!(vec![Some(1), Some(2)], lines);
}

#[test]
fn given_static_safe_and_unsafe_ranges_when_published_then_reports_only_potential_panics() {
    let mut context = read_code_context(
        "pub fn empty(values: &[u8]) -> &[u8] { &values[..0] }\npub fn empty_explicit(values: &[u8]) -> &[u8] { &values[0..0] }\npub fn full_explicit(values: &[u8]) -> &[u8] { &values[0..] }\npub fn prefix(values: &[u8; 4]) -> &[u8] { &values[..2] }\npub fn middle(values: &[u8; 4]) -> &[u8] { &values[1..=2] }\npub fn suffix(values: &[u8; 4]) -> &[u8] { &values[2..] }\npub fn outside(values: &[u8; 4]) -> &[u8] { &values[1..=4] }\npub fn reversed(values: &[u8; 4]) -> &[u8] { &values[3..1] }\npub fn maybe_empty(values: &[u8]) -> &[u8] { &values[..=0] }",
    );
    context.workspace.members[0].manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[package]\nname=\"case-sample\"\nversion=\"0.1.0\"",
    )
    .expect("published manifest");
    let lines: Vec<_> = DocCommentScope
        .check(&context)
        .iter()
        .map(|finding| finding.finding.location.line)
        .collect();
    assert_eq!(vec![Some(7), Some(8), Some(9)], lines);
}

#[test]
fn given_assertion_macros_and_examples_labels_when_published_then_requires_headed_contracts() {
    let mut context = read_code_context(
        "pub fn first(value: u64) { assert_eq!(value, 1); }\npub fn second(value: u64) { std::assert_ne!(value, 0); }\n/// # Panics\n/// Panics for zero.\npub fn valid(value: u64) { assert_ne!(value, 0); }\n/// Examples: free text.\npub fn wrong() {}\n/// # Examples\n/// ```\n/// assert_eq!(1 + 1, 2);\n/// ```\npub fn example() {}",
    );
    context.workspace.members[0].manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[package]\nname=\"case-sample\"\nversion=\"0.1.0\"",
    )
    .expect("published manifest");
    let lines: Vec<_> = DocCommentScope
        .check(&context)
        .iter()
        .map(|finding| finding.finding.location.line)
        .collect();
    assert_eq!(vec![Some(1), Some(2), Some(6)], lines);
}

#[test]
fn given_application_assertions_and_custom_macros_when_checked_then_does_not_invent_panic_contracts()
 {
    let context =
        read_code_context("fn first(value: u64) { assert_eq!(value, 1); assert_ne!(value, 0); }");
    assert!(DocCommentScope.check(&context).is_empty());
}

#[test]
fn given_aliased_standard_and_external_assertions_when_published_then_uses_macro_identity() {
    let mut context = read_code_context(
        "use std::assert_eq as ensure;\npub fn check(value: u64) { ensure!(value, 1); }\nmod custom { use tracing::assert_eq; pub fn check(value: u64) { assert_eq!(value, 1); } }\nmod local { macro_rules! assert_ne { ($value:expr, $other:expr) => {}; } pub fn check() { assert_ne!(1, 2); } }",
    );
    context.workspace.members[0].manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[package]\nname=\"case-sample\"\nversion=\"0.1.0\"",
    )
    .expect("published manifest");
    let lines: Vec<_> = DocCommentScope
        .check(&context)
        .iter()
        .map(|finding| finding.finding.location.line)
        .collect();
    assert_eq!(vec![Some(2)], lines);
}

#[test]
fn given_tdd_doc_panics_public_vector_indexing_when_published_then_reports_function_nodes() {
    let mut context = read_code_context(
        "pub fn element(values: Vec<u8>, index: usize) -> u8 { values[index] }\npub struct Reader { bytes: Vec<u8> }\nimpl Reader { pub fn read(&self, index: usize) -> u8 { self.bytes[index] } }",
    );
    context.workspace.members[0].manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[package]\nname=\"case-sample\"\nversion=\"0.1.0\"",
    )
    .expect("published manifest");
    let findings = DocCommentScope.check(&context);
    let actual: Vec<_> = findings
        .iter()
        .map(|finding| {
            assert_eq!("DocCommentScope", finding.finding.rule_id.0);
            let Anchor::Node { span, .. } = &finding.anchor else {
                panic!("expected function node anchor");
            };
            assert_eq!(finding.finding.location.line, Some(span.start.line));
            span.start.line
        })
        .collect();
    assert_eq!(vec![1, 3], actual);
}

fn assert_tdd_published_doc_examples_findings(source: &str) {
    let mut context = read_code_context(source);
    context.workspace.members[0].manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[package]\nname=\"case-sample\"\nversion=\"0.1.0\"",
    )
    .expect("published manifest");
    let findings = DocCommentScope.check(&context);
    assert_eq!(1, findings.len());
    assert_eq!("DocCommentScope", findings[0].finding.rule_id.0);
    assert_eq!(Some(1), findings[0].finding.location.line);
}

#[test]
fn given_tdd_doc_examples_ignore_fence_when_published_then_reports_heading() {
    assert_tdd_published_doc_examples_findings(
        "/// # Examples\n/// ```ignore\n/// let value = 1;\n/// ```\npub fn example() {}",
    );
}

#[test]
fn given_tdd_doc_examples_no_run_fence_when_published_then_reports_heading() {
    assert_tdd_published_doc_examples_findings(
        "/// # Examples\n/// ```no_run\n/// let value = 1;\n/// ```\npub fn example() {}",
    );
}

#[test]
fn given_tdd_doc_examples_non_rust_fence_when_published_then_reports_heading() {
    assert_tdd_published_doc_examples_findings(
        "/// # Examples\n/// ```text\n/// cargo run\n/// ```\npub fn example() {}",
    );
}

#[test]
fn given_tdd_doc_examples_runnable_and_disabled_fences_when_published_then_reports_heading() {
    assert_tdd_published_doc_examples_findings(
        "/// # Examples\n/// ```rust\n/// assert_eq!(2, 1 + 1);\n/// ```\n/// ```rust,ignore\n/// panic!();\n/// ```\n/// ```no_run\n/// let value = 1;\n/// ```\npub fn example() {}",
    );
}

#[test]
fn given_indented_valid_example_and_ignored_example_when_published_then_reports_only_invalid_heading()
 {
    let mut context = read_code_context(
        "/// # Examples\n///\n///     assert_eq!(1 + 1, 2);\npub fn valid() {}\n/// # Examples\n/// ```ignore\n/// assert_eq!(1 + 1, 2);\n/// ```\npub fn invalid() {}",
    );
    context.workspace.members[0].manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[package]\nname=\"case-sample\"\nversion=\"0.1.0\"",
    )
    .expect("published manifest");
    let findings = DocCommentScope.check(&context);
    assert_eq!(1, findings.len());
    assert_eq!("DocCommentScope", findings[0].finding.rule_id.0);
    assert_eq!(Some(5), findings[0].finding.location.line);
    assert!(matches!(&findings[0].anchor, Anchor::Node { .. }));
}

fn assert_published_scope_lines(source: &str, expected: &[u32]) {
    let mut context = read_code_context(source);
    context.workspace.members[0].manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[package]\nname=\"case-sample\"\nversion=\"0.1.0\"",
    )
    .expect("published manifest");
    let findings = DocCommentScope.check(&context);
    let mut actual: Vec<u32> = findings
        .iter()
        .map(|finding| {
            assert_eq!("DocCommentScope", finding.finding.rule_id.0);
            finding.finding.location.line.expect("finding line")
        })
        .collect();
    actual.sort_unstable();
    assert_eq!(expected, actual, "source: {source}\nfindings: {findings:?}");
}

#[test]
fn given_published_str_slice_by_parameter_index_when_checked_then_requires_panic_contract() {
    assert_published_scope_lines(
        "pub fn read_suffix(text: &str, index: usize) -> &str { &text[index..] }\npub fn whole(text: &str) -> &str { &text[..] }",
        &[1],
    );
}

#[test]
fn given_indented_panics_line_inside_doc_attribute_code_block_when_published_then_requires_panic_heading()
 {
    assert_published_scope_lines(
        "#[doc = \"Checks input.\\n\\n    struct Panics;\\n    let _ = {\\n    # Panics\\n    };\\n\"]\npub fn verify(value: bool) { assert!(value); }\n#[doc = \"Checks input.\\n\\n# Panics\\n\\nPanics when the value is false.\"]\npub fn verified(value: bool) { assert!(value); }",
        &[1],
    );
}

#[test]
fn given_trait_impl_method_asserting_caller_state_when_published_then_requires_panic_contract() {
    assert_published_scope_lines(
        "pub struct Counter { pub ready: bool }\npub trait Run { fn run(&self); }\nimpl Run for Counter {\nfn run(&self) { assert!(self.ready); }\n}\npub struct Gate { pub ready: bool }\nimpl Run for Gate {\n/// # Panics\n///\n/// Panics when the gate is not ready.\nfn run(&self) { assert!(self.ready); }\n}",
        &[4],
    );
}

#[test]
fn given_integer_division_by_parameter_when_published_then_requires_panic_contract() {
    assert_published_scope_lines(
        "pub fn share(total: u32, parts: u32) -> u32 { total / parts }\npub fn remainder(total: u64, size: u64) -> u64 { total % size }\npub fn half(total: u32) -> u32 { total / 2 }\npub fn ratio(total: f64, parts: f64) -> f64 { total / parts }",
        &[1, 2],
    );
}

#[test]
fn given_assertion_inside_closure_argument_when_published_then_requires_panic_contract() {
    assert_published_scope_lines(
        "pub fn total_within_limit(items: &[u32]) -> u32 {\n    items.iter().map(|item| { assert!(*item <= 100, \"item above limit\"); *item }).sum()\n}\npub fn checker() -> impl Fn(u32) {\n    |value| assert!(value > 0)\n}",
        &[1],
    );
}

#[test]
fn given_unwrap_err_and_panicking_fallback_closure_on_parameters_when_published_then_requires_panic_contract()
 {
    assert_published_scope_lines(
        "pub fn failure_of(result: Result<u32, String>) -> String { result.unwrap_err() }\npub fn rate_of(rate: Option<f64>) -> f64 { rate.unwrap_or_else(|| panic!(\"rate is required\")) }\npub fn expected_failure(result: Result<u32, String>) -> String { result.expect_err(\"must fail\") }\npub fn fallback(rate: Option<f64>) -> f64 { rate.unwrap_or_else(|| 0.0) }\npub fn local() -> String { let result: Result<u32, String> = Ok(1); result.unwrap_err() }",
        &[1, 2, 3],
    );
}

#[test]
fn given_assertion_inside_called_nested_function_when_published_then_requires_panic_contract() {
    assert_published_scope_lines(
        "pub fn guarded(value: u32) {\n    fn guard(value: u32) { assert!(value > 0); }\n    guard(value)\n}\npub fn unused(value: u32) {\n    fn guard(value: u32) { assert!(value > 0); }\n}",
        &[1],
    );
}
