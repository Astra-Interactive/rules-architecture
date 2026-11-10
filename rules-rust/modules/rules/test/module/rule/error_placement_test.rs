use crate::check::api::Rule;
use crate::config::model::Role;
use crate::module::rule::ErrorPlacement;
use crate::test::declaration::util::assert_code_fixture;
use crate::test::declaration::util::read_code_context;
use crate::test::module::util::assert_placement_findings;
use crate::workspace::model::{Dependency, DependencyKind};

#[test]
fn given_entry_route_returning_a_foreign_model_error_alias_when_checked_then_requires_an_entry_model_family()
 {
    let mut context = read_code_context(
        "mod route { type Reply = Result<(), tokio::model::LoadError>; fn load() -> Reply { loop {} } }",
    );
    let mut provider = read_code_context("pub mod model { pub enum LoadError { Missing } }");
    let mut member = provider.workspace.members.remove(0);
    member.name = "case-provider".to_owned();
    member.directory = "modules/provider".to_owned();
    let mut index = provider.indexes.remove(0);
    index.crate_name = member.name.clone();
    for module in &mut index.modules {
        module.module.crate_name = member.name.clone();
    }
    context.workspace.members[0].dependencies.push(Dependency {
        name: member.name.clone(),
        code_name: "tokio".to_owned(),
        path: Some(
            context
                .workspace
                .root
                .join(&member.directory)
                .to_string_lossy()
                .into_owned(),
        ),
        kind: DependencyKind::Normal,
        features: Vec::new(),
        is_target_specific: false,
        line: Some(1),
    });
    context.workspace.members.push(member);
    context.indexes.push(index);
    context.project_config.crates[0].role = Some(Role::EntryPoint);
    assert_placement_findings(
        &ErrorPlacement,
        "ErrorPlacement",
        &context,
        &[("src/lib.rs", 1, "Node")],
    );
    assert!(
        ErrorPlacement.check(&context)[0]
            .finding
            .subject
            .starts_with("case_provider::")
    );
    context.project_config.crates[0].role = Some(Role::Context);
    assert_placement_findings(&ErrorPlacement, "ErrorPlacement", &context, &[]);
}

#[test]
fn given_hand_written_error_family_when_declared_outside_model_then_reports_it() {
    let family = "enum SaveFailure { Disk } impl std::fmt::Display for SaveFailure { fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Ok(()) } } impl std::error::Error for SaveFailure {}";
    let context = read_code_context(&format!(
        "mod usecase {{ {family} }}\nmod model {{ {family} }}"
    ));
    assert_placement_findings(
        &ErrorPlacement,
        "ErrorPlacement",
        &context,
        &[("src/lib.rs", 1, "Node")],
    );
}

#[test]
fn given_types_implementing_other_traits_named_error_when_checked_then_only_std_error_reports() {
    let context = read_code_context(
        "mod contract { pub(crate) trait Error {} }\nmod usecase { pub(crate) enum SaveFailure { Disk } impl std::error::Error for SaveFailure {} }\nmod probing { use crate::contract::Error; pub(crate) struct Probe; impl Error for Probe {} }\nmod decoding { use vendor::de::Error; pub(crate) struct Decoder; impl Error for Decoder {} }",
    );
    assert_placement_findings(
        &ErrorPlacement,
        "ErrorPlacement",
        &context,
        &[("src/lib.rs", 2, "Node")],
    );
}

#[test]
fn given_persistent_placement_cases_when_checked_then_match_exact_node_anchors() {
    assert_code_fixture(&ErrorPlacement, "ErrorPlacement", "bad");
    assert_code_fixture(&ErrorPlacement, "ErrorPlacement", "good");
}
