use crate::module::rule::InMemoryPlacement;
use crate::test::declaration::util::{assert_code_findings, assert_code_fixture};

#[test]
fn given_persistent_placement_cases_when_checked_then_match_exact_node_anchors() {
    assert_code_fixture(&InMemoryPlacement, "InMemoryPlacement", "bad");
    assert_code_fixture(&InMemoryPlacement, "InMemoryPlacement", "good");
}

#[test]
fn given_in_memory_enums_when_checked_then_misnamed_and_misplaced_enums_are_reported() {
    assert_code_findings(
        &InMemoryPlacement,
        "InMemoryPlacement",
        "mod api { pub trait Repository {} }\nmod memory {\n    enum MemoryRepository { Empty } impl crate::api::Repository for MemoryRepository {}\n    enum InMemoryRepository { Empty } impl crate::api::Repository for InMemoryRepository {}\n}\nmod util { enum InMemoryCache { Empty } }",
        &[3, 6],
    );
}
