use crate::declaration::rule::ModelDerives;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_nominal_copy_identifier_when_checked_then_derive_and_explicit_impl_require_copy() {
    assert_code_findings(
        &ModelDerives,
        "ModelDerives",
        "#[derive(Copy, Clone)] struct Number(u64);\nstruct Manual(u64); impl Copy for Manual {} impl Clone for Manual { fn clone(&self) -> Self { *self } }\nmod model {\n#[derive(Debug, Clone, PartialEq, Eq, Hash)] struct UserId(crate::Number);\n#[derive(Debug, Clone, PartialEq, Eq, Hash)] struct ManualId(crate::Manual);\n#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] struct ValidId(crate::Number);\n}\n",
        &[4, 5],
    );
}

#[test]
fn given_uuid_identifier_when_checked_then_resolved_external_copy_is_required() {
    assert_code_findings(
        &ModelDerives,
        "ModelDerives",
        "use uuid::Uuid as External; mod model {\n#[derive(Debug, Clone, PartialEq, Eq, Hash)] struct UserId(super::External);\n#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] struct ValidId(uuid::Uuid);\n#[derive(Debug, Clone, PartialEq, Eq, Hash)] struct OtherId(vendor::Uuid);\n}",
        &[2],
    );
}

#[test]
fn given_renamed_uuid_dependency_when_checked_then_package_identity_proves_copy() {
    use crate::check::api::Rule;
    let mut context = crate::test::declaration::util::read_code_context(
        "mod model { #[derive(Debug, Clone, PartialEq, Eq, Hash)] struct UserId(identifier::Uuid); }",
    );
    context.indexes[0]
        .dependencies
        .push("identifier".to_owned());
    context.workspace.members[0].name = context.indexes[0].crate_name.clone();
    context.workspace.members[0]
        .dependencies
        .push(crate::workspace::model::Dependency {
            name: "uuid".to_owned(),
            code_name: "identifier".to_owned(),
            path: None,
            kind: crate::workspace::model::DependencyKind::Normal,
            features: Vec::new(),
            is_target_specific: false,
            line: None,
        });
    let findings = ModelDerives.check(&context);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].finding.message.contains("Copy"));
}

#[test]
fn given_local_uuid_and_cfg_copy_types_when_checked_then_requires_copy_only_where_proven() {
    assert_code_findings(
        &ModelDerives,
        "ModelDerives",
        "mod uuid { pub struct Uuid; } #[cfg(unix)] #[derive(Copy)] struct Inner; #[cfg(not(unix))] struct Inner;\nmod model {\n#[derive(Debug, Clone, PartialEq, Eq, Hash)] struct LocalId(crate::uuid::Uuid);\n#[cfg(unix)] #[derive(Debug, Clone, PartialEq, Eq, Hash)] struct UnixId(crate::Inner);\n#[cfg(not(unix))] #[derive(Debug, Clone, PartialEq, Eq, Hash)] struct OtherId(crate::Inner);\n}",
        &[4],
    );
}

#[test]
fn given_conditional_copy_impl_when_checked_then_uncovered_cfg_does_not_prove_copy() {
    assert_code_findings(
        &ModelDerives,
        "ModelDerives",
        "struct Value; #[cfg(unix)] impl Copy for Value {}\nmod model { #[derive(Debug, Clone, PartialEq, Eq, Hash)] struct UserId(crate::Value); }",
        &[],
    );
}

#[test]
fn given_identifier_copy_support_when_checked_then_requires_copy_only_for_proven_inner_types() {
    assert_code_findings(
        &ModelDerives,
        "ModelDerives",
        "type Number = u64; mod model {\n#[derive(Debug, Clone, PartialEq, Eq, Hash)] struct UserId(u64);\n#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] struct GroupId(u64);\n#[derive(Debug, Clone, PartialEq, Eq, Hash)] struct NameId(String);\n#[derive(Debug, Clone, PartialEq, Eq, Hash)] struct ExternalId(external::Value);\n#[derive(Debug, Clone, PartialEq, Eq, Hash)] struct AliasId(crate::Number);\n}",
        &[2, 6],
    );
}

#[test]
fn given_persistent_code_fixtures_when_checked_then_match_exact_anchors() {
    assert_code_fixture(&ModelDerives, "ModelDerives", "bad");
    assert_code_fixture(&ModelDerives, "ModelDerives", "good");
}

#[test]
fn given_forbidden_code_when_checked_then_reports_exact_node_lines() {
    assert_code_findings(
        &ModelDerives,
        "ModelDerives",
        "mod model {\n    struct User { name: String }\n    #[derive(Debug, Clone, PartialEq)] struct UserId(u64);\n}",
        &[2, 3],
    );
}

#[test]
fn given_prescribed_code_when_checked_then_passes() {
    assert_code_findings(
        &ModelDerives,
        "ModelDerives",
        "mod model {\n    #[derive(Debug, Clone, PartialEq, Eq, Hash)] struct User { name: String }\n    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] struct UserId(u64);\n}\nmod file { struct Row; }",
        &[],
    );
}

#[test]
fn given_eq_hash_field_support_when_model_checked_then_float_and_unproved_external_types_pass() {
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ModelDerives,
        "mod model {\n#[derive(Debug, Clone, PartialEq)] struct Count(u64);\n#[derive(Debug, Clone, PartialEq)] struct Ratio(f64);\n#[derive(Debug, Clone, PartialEq)] struct Foreign(std::io::Error);\n}",
        &[2],
    );
}
