use crate::check::model::RuleContext;
use crate::config::model::Role;
use crate::module::rule::EventPlacement;
use crate::source::syn::CrateIndexBuilder;
use crate::test::declaration::util::assert_code_fixture;
use crate::test::declaration::util::read_code_context;
use crate::test::module::util::add_workspace_provider;
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
fn given_platform_words_when_event_derives_or_implements_a_non_host_trait_then_reports_it() {
    let mut context = read_code_context(
        "mod event {\n    use serde::Serialize;\n    use vendor::Event;\n    #[derive(Serialize)]\n    struct PlayerJailedEvent(u32);\n    #[derive(Event, Serialize)]\n    struct PlayerJoinEvent(u32);\n    struct PlayerQuitEvent(u32);\n    impl tokio::Marker for PlayerQuitEvent {}\n    struct PlayerMoveEvent(u32);\n    impl vendor::HostEvent for PlayerMoveEvent {}\n}",
    );
    context.project_config.crates[0].role = Some(Role::Platform);
    context.project_config.platform_words = vec!["vendor".to_owned()];
    assert_placement_findings(
        &EventPlacement,
        "EventPlacement",
        &context,
        &[("src/lib.rs", 4, "Node"), ("src/lib.rs", 8, "Node")],
    );
    context.project_config.platform_words = Vec::new();
    assert_placement_findings(&EventPlacement, "EventPlacement", &context, &[]);
}

fn read_platform_context_with_workspace_provider() -> RuleContext {
    let mut context = read_code_context(
        "mod event {\n    use case_provider::Event;\n    #[derive(Event)]\n    struct PlayerJoinEvent(u32);\n    struct PlayerQuitEvent(u32);\n    impl case_provider::HostSignal for PlayerQuitEvent {}\n}",
    );
    let mut dependencies = context.indexes[0].dependencies.clone();
    dependencies.push("case_provider".to_owned());
    context.indexes[0] = CrateIndexBuilder::new(&context.crates[0], dependencies).build();
    add_workspace_provider(
        &mut context,
        "pub trait HostSignal {}\n#[proc_macro_derive(Event)]\npub fn derive_event() {}",
        "case_provider",
    );
    context.project_config.crates[0].role = Some(Role::Platform);
    context
}

#[test]
fn given_platform_words_when_event_uses_derive_or_trait_of_another_workspace_crate_then_reports_both()
 {
    let mut context = read_platform_context_with_workspace_provider();
    context.project_config.platform_words = vec!["vendor".to_owned()];
    assert_placement_findings(
        &EventPlacement,
        "EventPlacement",
        &context,
        &[("src/lib.rs", 3, "Node"), ("src/lib.rs", 5, "Node")],
    );
}

#[test]
fn given_no_platform_words_when_event_uses_derive_or_trait_of_another_workspace_crate_then_reports_both()
 {
    let mut context = read_platform_context_with_workspace_provider();
    context.project_config.platform_words = Vec::new();
    assert_placement_findings(
        &EventPlacement,
        "EventPlacement",
        &context,
        &[("src/lib.rs", 3, "Node"), ("src/lib.rs", 5, "Node")],
    );
}

#[test]
fn given_persistent_placement_cases_when_checked_then_match_exact_node_anchors() {
    assert_code_fixture(&EventPlacement, "EventPlacement", "bad");
    assert_code_fixture(&EventPlacement, "EventPlacement", "good");
}
