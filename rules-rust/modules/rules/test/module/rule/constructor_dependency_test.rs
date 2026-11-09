#[test]
fn given_real_cargo_rename_and_reexport_when_dependency_constructed_then_identity_and_statement_anchor_match()
 {
    use crate::source::api::SourceReader;
    use crate::workspace::api::WorkspaceReader;
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../cases/code/ConstructorDependency/workspace");
    let mut context = crate::test::declaration::util::read_code_context("");
    context.workspace = crate::workspace::cargo::CargoWorkspaceReader::new(
        crate::workspace::util::resolve_cargo_path(),
    )
    .read_workspace(&directory)
    .expect("Cargo workspace");
    context.project_config =
        crate::config::util::parse_project_config(&context.workspace).expect("configuration");
    context.crates = crate::source::syn::SynSourceReader.read_sources(&context.workspace);
    context.indexes =
        crate::source::syn::CrateIndexBuilder::build_all(&context.crates, &context.workspace);
    assert!(
        context
            .indexes
            .iter()
            .all(|index| index.unanalysable.is_empty())
    );
    let findings =
        crate::check::api::Rule::check(&crate::module::rule::ConstructorDependency, &context);
    let actual = findings
        .iter()
        .map(|finding| {
            format!(
                "{}\t{}:{}\tNode\t{}",
                finding.finding.rule_id.0,
                finding.finding.location.path,
                finding.finding.location.line.expect("line"),
                finding.finding.subject
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let expected = std::fs::read_to_string(directory.join("expected.tsv")).expect("exact findings");
    assert_eq!(expected.trim_end(), actual);
}

#[test]
fn given_workspace_constructors_when_checked_then_own_state_and_di_are_exempt_and_other_implementations_fail()
 {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::ConstructorDependency,
        "trait Repo {} struct Concrete; impl Repo for Concrete {} impl Concrete { fn new() -> Self { Self } }\n struct Consumer; impl Consumer { fn new() -> Self { let repo = Concrete::new(); Self } }",
        &[2],
    );
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::ConstructorDependency,
        "trait Repo {} struct Concrete; impl Repo for Concrete {} impl Concrete { fn new() -> Self { Concrete } fn helper(&self) {} }\nmod di { struct OwnerModule; impl OwnerModule { fn new() -> Self { let repo = super::Concrete::new(); Self } } }",
        &[],
    );
}

#[test]
fn given_persistent_di_policy_pairs_when_checked_then_exact_nodes_and_subjects_match() {
    for half in ["bad", "good"] {
        crate::test::module::util::assert_di_fixture(
            &crate::module::rule::ConstructorDependency,
            "ConstructorDependency",
            half,
        );
    }
}
#[test]
fn given_known_factory_constructors_when_checked_then_dependencies_fail_and_local_state_passes() {
    for constructor in ["with_config", "from_config", "default"] {
        crate::test::module::util::assert_di_lines(
            &crate::module::rule::ConstructorDependency,
            &format!(
                "struct RemoteClient; impl RemoteClient {{ fn {constructor}() -> Self {{ Self }} fn send(&self) {{}} }}\nstruct Consumer; impl Consumer {{ fn new() -> Self {{ let client = RemoteClient::{constructor}(); Self }} }}"
            ),
            &[2],
        );
    }
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::ConstructorDependency,
        "struct State; impl State { fn new() -> Self { Self } fn update(&mut self) {} } struct Consumer; impl Consumer { fn new() -> Self { let state = State::new(); let values = Vec::<u8>::new(); Self } }",
        &[],
    );
}

#[test]
fn given_host_composition_root_and_packet_representation_when_checked_then_neither_is_a_service_dependency()
 {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::ConstructorDependency,
        "mod di { pub struct RootModule; impl RootModule { pub fn new() -> Self { Self } } } struct Plugin; impl Plugin { fn on_load(&self) { let root = di::RootModule::new(); } } trait ClientboundPacket { fn encode(&self); } struct LoginDisconnect; impl ClientboundPacket for LoginDisconnect { fn encode(&self) {} } impl LoginDisconnect { fn new() -> Self { Self } } struct Session; impl Session { fn send(&self) { let packet = LoginDisconnect::new(); } }",
        &[],
    );
}
