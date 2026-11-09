use crate::check::api::Rule;
use crate::comment::rule::DocCommentScope;
use crate::test::declaration::util::read_code_context;

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
