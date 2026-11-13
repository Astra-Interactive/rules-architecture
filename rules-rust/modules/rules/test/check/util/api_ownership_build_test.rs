use crate::check::util::build_api_ownership;
use crate::config::model::Role;
use crate::test::declaration::util::read_code_context;
use crate::test::source::util::read_resolution_context;

#[test]
fn given_literal_inherited_or_missing_version_when_advised_then_only_publishable_packages_are_external()
 {
    for (package, workspace, external) in [
        ("", "", false),
        ("publish=true", "", false),
        ("version=\"1.0.0\"", "", true),
        ("", "[workspace.package]\nversion=\"1.0.0\"", false),
        (
            "version.workspace=true",
            "[workspace.package]\nversion=\"1.0.0\"",
            true,
        ),
        ("version=\"1.0.0\"\npublish=false", "", false),
        ("version=\"1.0.0\"\npublish=[]", "", false),
        ("version=\"1.0.0\"\npublish=[\"internal\"]", "", true),
        (
            "version.workspace=true\npublish.workspace=true",
            "[workspace.package]\nversion=\"1.0.0\"\npublish=false",
            false,
        ),
    ] {
        let mut context = read_code_context("pub struct Value;");
        context.project_config.crates[0].role = Some(Role::Api);
        context.workspace.members[0].manifest = crate::workspace::util::parse_toml_file(
            "Cargo.toml",
            &format!("[package]\nname=\"case-sample\"\n{package}\n"),
        )
        .unwrap();
        context.workspace.root_manifest = crate::workspace::util::parse_toml_file(
            "Cargo.toml",
            &format!("[workspace]\nresolver=\"3\"\n{workspace}\n"),
        )
        .unwrap();
        let advice = build_api_ownership(&context);
        assert_eq!(
            external,
            advice.as_text().contains("EXTERNAL\t"),
            "{package}; {workspace}"
        );
        assert_eq!(
            !external,
            context.can_prove_workspace_only(0),
            "{package}; {workspace}"
        );
    }
}

#[test]
fn given_sibling_implementation_and_unused_api_when_advised_then_candidate_and_unknown_remain_non_failing_evidence()
 {
    let mut context = read_resolution_context();
    for config in &mut context.project_config.crates {
        config.role = Some(if config.name == "digest-package" {
            Role::Api
        } else {
            Role::Impl
        });
    }
    let text = build_api_ownership(&context);
    assert!(
        text.as_text().contains("CANDIDATE\tdigest_package::Digest"),
        "{}",
        text.as_text()
    );
    assert!(text.as_text().contains("UNKNOWN\tdigest_package::Unused"));
    assert!(text.as_text().contains("UNKNOWN\tdigest_package::TestOnly"));
    assert!(
        text.as_text()
            .contains("UNKNOWN\tdigest_package::BranchModel")
    );
    for config in &mut context.project_config.crates {
        if config.name == "identity-owner" {
            config.role = Some(Role::Context);
        }
    }
    assert!(
        build_api_ownership(&context)
            .as_text()
            .contains("RETAIN\tdigest_package::Digest")
    );
}

#[test]
fn given_published_or_unanalysable_api_when_advised_then_absent_names_do_not_prove_single_owner() {
    let mut context = read_code_context("pub struct Value;");
    context.project_config.crates[0].role = Some(Role::Api);
    context.workspace.members[0].manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[package]\nname=\"case-sample\"\nversion=\"1.0.0\"\n",
    )
    .unwrap();
    assert!(
        build_api_ownership(&context)
            .as_text()
            .contains("EXTERNAL\t")
    );
    let name = context.workspace.members[0].name.clone();
    context.workspace.members[0].manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        &format!("[package]\nname={name:?}\npublish=false\n"),
    )
    .unwrap();
    assert!(
        build_api_ownership(&context)
            .as_text()
            .contains("UNKNOWN\t")
    );
    let manifest = context.workspace.members[0].manifest.clone();
    let mut context = read_code_context(
        "pub struct Value; macro_rules! make_consumer { () => { fn consume(_:crate::Value){} }; } mod consumer { make_consumer!(); }",
    );
    context.project_config.crates[0].role = Some(Role::Api);
    context.workspace.members[0].manifest = manifest;
    assert!(
        build_api_ownership(&context)
            .as_text()
            .contains("UNCERTAIN\t")
    );
}
