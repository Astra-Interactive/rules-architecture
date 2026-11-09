use crate::declaration::rule::ModelDerives;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

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
