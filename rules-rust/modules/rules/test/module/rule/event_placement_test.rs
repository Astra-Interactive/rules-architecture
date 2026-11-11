use crate::config::model::Role;
use crate::module::rule::EventPlacement;
use crate::test::declaration::util::assert_code_fixture;
use crate::test::declaration::util::read_code_context;
use crate::test::module::util::assert_placement_findings;

#[test]
fn given_external_event_impl_when_checked_then_exempts_only_platform_event_types() {
    let mut context = read_code_context(
        "mod event { use tokio::HostEvent as External; struct PlayerEvent; type Alias = PlayerEvent; impl External for Alias {} }",
    );
    context.project_config.crates[0].role = Some(Role::Platform);
    assert_placement_findings(&EventPlacement, "EventPlacement", &context, &[]);
    context.project_config.crates[0].role = Some(Role::Context);
    assert_placement_findings(
        &EventPlacement,
        "EventPlacement",
        &context,
        &[("src/lib.rs", 1, "Node")],
    );
    let mut context = read_code_context(
        "mod api { trait HostContract {} } mod event { struct PlayerEvent; impl crate::api::HostContract for PlayerEvent {} }",
    );
    context.project_config.crates[0].role = Some(Role::Platform);
    assert_placement_findings(
        &EventPlacement,
        "EventPlacement",
        &context,
        &[("src/lib.rs", 1, "Node")],
    );
}

#[test]
fn given_platform_event_whose_host_trait_is_derived_when_checked_then_only_the_plain_one_reports() {
    let mut context = read_code_context(
        "mod event {\n    use vendor::Event;\n    #[derive(Event)]\n    struct PlayerJoinEvent(u32);\n    #[derive(Debug, Clone)]\n    struct PlayerQuitEvent(u32);\n}",
    );
    context.project_config.crates[0].role = Some(Role::Platform);
    assert_placement_findings(
        &EventPlacement,
        "EventPlacement",
        &context,
        &[("src/lib.rs", 5, "Node")],
    );
}

#[test]
fn given_persistent_placement_cases_when_checked_then_match_exact_node_anchors() {
    assert_code_fixture(&EventPlacement, "EventPlacement", "bad");
    assert_code_fixture(&EventPlacement, "EventPlacement", "good");
}
