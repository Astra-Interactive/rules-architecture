use crate::module::rule::ListenerName;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_non_listener_event_types_when_implementing_standard_traits_then_keeps_their_names() {
    assert_code_findings(
        &ListenerName,
        "ListenerName",
        "mod event {\nstruct Subscription;\nimpl Drop for Subscription { fn drop(&mut self) {} }\nstruct MessageEvent;\nimpl std::fmt::Display for MessageEvent {}\nstruct Observer;\nimpl tokio::Listener for Observer {}\n}",
        &[6],
    );
}

#[test]
fn given_persistent_placement_cases_when_checked_then_match_exact_node_anchors() {
    assert_code_fixture(&ListenerName, "ListenerName", "bad");
    assert_code_fixture(&ListenerName, "ListenerName", "good");
}
