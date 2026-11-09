use crate::test::declaration::util::assert_code_findings;
use crate::tuple::rule::AnonymousTuple;

#[test]
fn given_standard_enumerate_and_partition_when_bound_then_requires_immediate_destructuring() {
    assert_code_findings(
        &AnonymousTuple,
        "AnonymousTuple",
        "fn run(values: &[u8], custom: Custom) {\nfor entry in values.iter().enumerate() {}\nfor (index, value) in values.iter().enumerate() {}\nlet parts = values.iter().partition::<Vec<_>, _>(|value| **value > 0);\nlet (left, right) = values.iter().partition::<Vec<_>, _>(|value| **value > 0);\nfor entry in custom.enumerate() {}\nlet named = custom.partition();\n}\nstruct Custom; impl Custom { fn enumerate(&self) -> Vec<u8> { vec![] } fn partition(&self) -> u8 { 0 } }",
        &[2, 4],
    );
}

#[test]
fn given_pattern_scopes_when_checked_then_known_outer_receiver_types_are_restored() {
    assert_code_findings(
        &AnonymousTuple,
        "AnonymousTuple",
        "fn run(text: &str, value: Option<Buffer>) {\nfor text in [1] { let _ = text; }\nlet after_for = text.split_at(1);\nif let Some(text) = value { let custom = text.split_at(1); } else { let outer = text.split_at(1); }\nlet after_if = text.split_at(1);\nwhile let Some(text) = None::<Buffer> { let custom = text.split_at(1); break; }\nlet after_while = text.split_at(1);\nmatch None::<Buffer> { Some(text) => { let custom = text.split_at(1); }, None => { let outer = text.split_at(1); } }\nlet after_match = text.split_at(1);\n}\nstruct Buffer; impl Buffer { fn split_at(&self, index: usize) -> Named { Named } } struct Named;",
        &[3, 4, 5, 7, 8, 9],
    );
}

#[test]
fn given_proven_iterator_aliases_when_checked_then_tracks_tuple_results_and_lexical_shadowing() {
    assert_code_findings(
        &AnonymousTuple,
        "AnonymousTuple",
        "fn run(values: &[u8], custom: Custom) {\nlet iter = values.iter();\nfor entry in iter.enumerate() {}\nlet iter = values.iter();\nlet parts = iter.partition::<Vec<_>, _>(|value| **value > 0);\nlet iter = values.iter();\n{ let iter = custom; for entry in iter.enumerate() {} let named = iter.partition(); }\nfor entry in iter.enumerate() {}\nlet iter = values.iter().enumerate();\nlet alias = iter;\nfor entry in alias {}\nlet iter = custom;\nfor entry in iter.enumerate() {}\n}\nstruct Custom; impl Custom { fn enumerate(&self) -> Vec<u8> { vec![] } fn partition(&self) -> u8 { 0 } }",
        &[3, 5, 8, 11],
    );
}

#[test]
fn given_string_extension_iter_when_checked_then_does_not_assume_slice_iterator_methods() {
    assert_code_findings(
        &AnonymousTuple,
        "AnonymousTuple",
        "trait Bytes { fn iter(&self) -> Custom; } impl Bytes for str { fn iter(&self) -> Custom { Custom } }\nstruct Custom; impl Custom { fn enumerate(&self) -> Vec<u8> { vec![] } }\nfn run(text: &str) { for byte in text.iter().enumerate() {} let iter = text.iter(); for byte in iter.enumerate() {} }",
        &[],
    );
}

#[test]
fn given_collection_extension_override_when_checked_then_does_not_assume_dereferenced_std_methods()
{
    assert_code_findings(
        &AnonymousTuple,
        "AnonymousTuple",
        "trait Bytes { fn iter(&self) -> Custom; } impl Bytes for Vec<u8> { fn iter(&self) -> Custom { Custom } }\nstruct Custom; impl Custom { fn enumerate(&self) -> Vec<u8> { vec![] } }\nfn run(values: Vec<u8>) { for byte in values.iter().enumerate() {} let iter = values.iter(); for byte in iter.enumerate() {} }",
        &[],
    );
}

#[test]
fn given_condition_initializers_when_checked_then_bindings_start_after_initializer_evaluation() {
    assert_code_findings(
        &AnonymousTuple,
        "AnonymousTuple",
        "fn run(text: &str) {\nif let Some(text) = { let parts = text.split_at(1); None::<Buffer> } { let custom = text.split_at(1); }\nwhile let Some(text) = { let parts = text.split_at(1); None::<Buffer> } { let custom = text.split_at(1); break; }\nfor text in { let parts = text.split_at(1); [Buffer] } { let custom = text.split_at(1); }\n}\nstruct Buffer; impl Buffer { fn split_at(&self, index: usize) -> Named { Named } } struct Named;",
        &[2, 3, 4],
    );
}

#[test]
fn given_shadowed_function_names_when_checked_then_does_not_infer_an_unrelated_return_type() {
    assert_code_findings(
        &AnonymousTuple,
        "AnonymousTuple",
        "fn pair() -> (u8, u8) { (1, 2) }\nfn work(pair: fn() -> Named) { let named = pair(); }\nstruct Named;",
        &[1],
    );
}

#[test]
fn given_proven_tuple_values_when_aliased_then_tracks_values_without_crossing_shadow_scopes() {
    assert_code_findings(
        &AnonymousTuple,
        "AnonymousTuple",
        "fn pair() -> (u8, u8) { (1, 2) }\nfn work(text: &str) {\nlet parts = pair();\nlet copy = parts;\n{ let parts = Named; let named = parts; }\nlet copied = parts;\nlet alias = text;\nlet halves = alias.split_at(1);\nlet closure = |text: Buffer| { let parts = text.split_at(1); };\nlet final_parts = text.split_at(1);\n}\nstruct Named; struct Buffer; impl Buffer { fn split_at(&self, index: usize) -> Named { Named } }",
        &[1, 3, 4, 6, 8, 10],
    );
}

#[test]
fn given_typed_standard_receivers_when_checked_then_tuple_bindings_require_destructuring() {
    assert_code_findings(
        &AnonymousTuple,
        "AnonymousTuple",
        "fn work(text: &str, bytes: &[u8], buffer: Buffer) {\nlet parts = text.split_at(1);\nlet halves = bytes.split_at(1);\nlet custom = buffer.split_at(1);\nlet (left, right) = text.split_at(1);\n}\nstruct Buffer; impl Buffer { fn split_at(&self, index: usize) -> Parts { todo!() } } struct Parts;",
        &[2, 3],
    );
}
