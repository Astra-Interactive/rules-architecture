use crate::check::api::Rule;
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
        "pub fn first(value: u64) { assert_eq!(value, 1); }\npub fn second(value: u64) { std::assert_ne!(value, 0); }\n/// # Panics\n/// Panics for zero.\npub fn valid(value: u64) { assert_ne!(value, 0); }\n/// Examples: free text.\npub fn wrong() {}\n/// # Examples\n/// Useful usage.\npub fn example() {}",
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
