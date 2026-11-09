use crate::module::rule::InMemoryPlacement;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_persistent_placement_cases_when_checked_then_match_exact_node_anchors() {
    assert_code_fixture(&InMemoryPlacement, "InMemoryPlacement", "bad");
    assert_code_fixture(&InMemoryPlacement, "InMemoryPlacement", "good");
}
