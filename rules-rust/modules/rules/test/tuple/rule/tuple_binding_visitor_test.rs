use crate::test::declaration::util::assert_code_findings;
use crate::tuple::rule::AnonymousTuple;

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
