use crate::test::declaration::util::assert_code_findings;
use crate::tuple::rule::AnonymousTuple;

#[test]
fn given_tuple_from_a_std_method_chain_when_bound_to_one_name_then_requires_destructuring() {
    assert_code_findings(
        &AnonymousTuple,
        "AnonymousTuple",
        "fn work(text: &str) {\n    let parts = text.trim().split_at(3);\n    for pair in text.trim().char_indices() { let _position = pair.0; }\n    let (head, tail) = text.trim().split_at(3);\n    let custom = text.widen().split_at(3);\n    for (position, _char) in text.trim().char_indices() {}\n}\ntrait Widen { fn widen(&self) -> Buffer; } struct Buffer; impl Buffer { fn split_at(&self, index: usize) -> Parts { todo!() } } struct Parts;",
        &[2, 3],
    );
}

#[test]
fn given_tuple_bound_to_one_name_inside_a_pattern_when_checked_then_requires_destructuring() {
    assert_code_findings(
        &AnonymousTuple,
        "AnonymousTuple",
        "fn read_key(text: &str, pairs: &mut Vec<Pair>) -> Option<&str> {\n    if let Some(pair) = text.split_once('=') {\n        return Some(pair.0);\n    }\n    let Some(pair) = text.rsplit_once('=') else { return None; };\n    match text.split_once(':') {\n        Some(pair) => return Some(pair.0),\n        None => {}\n    }\n    if let Some((key, _value)) = text.split_once('=') { return Some(key); }\n    if let Some(pair) = pairs.pop() { return Some(pair.key); }\n    Some(pair.1)\n}\nstruct Pair { key: &'static str }",
        &[2, 5, 7],
    );
}

#[test]
fn given_split_first_and_its_siblings_when_bound_to_one_name_then_requires_destructuring() {
    assert_code_findings(
        &AnonymousTuple,
        "AnonymousTuple",
        "fn head_of(values: &[u32], text: &str) -> u32 {\n    let parts = values.split_first().expect(\"non-empty\");\n    let last = values.split_last().unwrap();\n    let halves = text.split_at_checked(2).expect(\"long enough\");\n    let (head, _tail) = values.split_first().expect(\"non-empty\");\n    let first = values.first().copied().unwrap_or_default();\n    *parts.0 + *last.0 + halves.0.len() as u32 + *head + first\n}",
        &[2, 3, 4],
    );
}

#[test]
fn given_tuple_bound_to_one_name_in_a_trait_default_method_when_checked_then_requires_destructuring()
 {
    assert_code_findings(
        &AnonymousTuple,
        "AnonymousTuple",
        "pub(crate) trait LineSplit {\n    fn first_line(&self) -> &str;\n    fn head_of(&self, text: &str) -> usize {\n        let parts = text.split_at(1);\n        parts.0.len()\n    }\n    fn tail_of(&self, text: &str) -> usize {\n        let (_head, tail) = text.split_at(1);\n        tail.len()\n    }\n}",
        &[4],
    );
}

#[test]
fn given_tuple_bound_to_one_name_in_a_select_handler_when_checked_then_requires_destructuring() {
    assert_code_findings(
        &AnonymousTuple,
        "AnonymousTuple",
        "pub(crate) async fn watch(mut tick: tokio::sync::mpsc::Receiver<u32>, text: &str) {\n    tokio::select! {\n        value = tick.recv() => {\n            let parts = text.split_at(1);\n            let (head, tail) = text.split_at(1);\n            drop(value);\n        }\n    }\n}",
        &[4],
    );
}

#[test]
fn given_tuple_bound_in_one_select_branch_when_another_binds_the_name_then_reports_one_line() {
    assert_code_findings(
        &AnonymousTuple,
        "AnonymousTuple",
        "pub(crate) async fn watch(mut tick: tokio::sync::mpsc::Receiver<u32>, mut stop: tokio::sync::mpsc::Receiver<u32>, text: &str) {\n    tokio::select! {\n        value = tick.recv() => {\n            let parts = text.split_at(1);\n            drop(value);\n        }\n        parts = stop.recv() => {\n            let copy = parts;\n        }\n    }\n}",
        &[4],
    );
}

#[test]
fn given_tuple_bound_before_a_select_when_a_branch_rebinds_the_name_then_other_branches_keep_it() {
    assert_code_findings(
        &AnonymousTuple,
        "AnonymousTuple",
        "pub(crate) async fn watch(mut tick: tokio::sync::mpsc::Receiver<u32>, mut stop: tokio::sync::mpsc::Receiver<u32>, text: &str) {\n    let parts = text.split_at(1);\n    tokio::select! {\n        parts = tick.recv() => {\n            drop(parts);\n        }\n        stopped = stop.recv() => {\n            let copy = parts;\n            drop(stopped);\n        }\n    }\n}",
        &[2, 8],
    );
}

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

#[test]
fn given_split_once_unwrapped_when_bound_to_a_name_then_requires_destructuring() {
    assert_code_findings(
        &AnonymousTuple,
        "AnonymousTuple",
        "fn run() {\nlet parts = \"a=b\".split_once('=').unwrap();\nlet (key, value) = \"a=b\".split_once('=').unwrap();\n}",
        &[2],
    );
}

#[test]
fn given_closure_parameter_bound_to_enumerate_item_when_checked_then_requires_destructuring() {
    assert_code_findings(
        &AnonymousTuple,
        "AnonymousTuple",
        "fn run(values: &[u8]) {\nlet named = values.iter().enumerate().map(|entry| entry.0);\nlet split = values.iter().enumerate().map(|(index, value)| index);\n}",
        &[2],
    );
}

#[test]
fn given_for_loop_over_zip_when_item_is_bound_to_a_name_then_requires_destructuring() {
    assert_code_findings(
        &AnonymousTuple,
        "AnonymousTuple",
        "fn run(prices: &[u8], quantities: &[u8]) {\nfor pair in prices.iter().zip(quantities.iter()) {}\nfor (price, quantity) in prices.iter().zip(quantities.iter()) {}\n}",
        &[2],
    );
}

#[test]
fn given_char_indices_and_map_iteration_when_item_is_bound_then_requires_destructuring() {
    assert_code_findings(
        &AnonymousTuple,
        "AnonymousTuple",
        "use std::collections::{BTreeMap, HashMap};\nfn run(text: &str, scores: &HashMap<String, u32>, ranks: &BTreeMap<String, u32>) {\nfor pair in text.char_indices() { let _ = pair.0; }\nfor entry in scores.iter() {}\nfor entry in ranks.iter() {}\nfor (index, character) in text.char_indices() {}\nfor (name, score) in scores.iter() {}\n}",
        &[3, 4, 5],
    );
}

#[test]
fn given_tuple_bound_through_branch_initializer_when_checked_then_requires_destructuring() {
    assert_code_findings(
        &AnonymousTuple,
        "AnonymousTuple",
        "fn run(wide: bool) {\nlet range = if wide { (0_u32, 100_u32) } else { (0_u32, 10_u32) };\nlet chosen = match wide { true => (0_u32, 100_u32), false => (0_u32, 10_u32) };\nlet block = { (0_u32, 10_u32) };\nlet (low, high) = if wide { (0, 100) } else { (0, 10) };\n}",
        &[2, 3, 4],
    );
}

#[test]
fn given_tdd_anonymous_tuple_size_hint_when_linted_then_reports_named_tuple_binding() {
    assert_code_findings(
        &AnonymousTuple,
        "AnonymousTuple",
        "fn run(values: &[u8]) -> usize {\nlet bounds = values.iter().size_hint();\nbounds.0\n}",
        &[2],
    );
}
