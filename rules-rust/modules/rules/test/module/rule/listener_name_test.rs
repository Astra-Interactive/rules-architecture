use crate::module::rule::ListenerName;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_persistent_placement_cases_when_checked_then_match_exact_node_anchors() {
    assert_code_fixture(&ListenerName, "ListenerName", "bad");
    assert_code_fixture(&ListenerName, "ListenerName", "good");
}
