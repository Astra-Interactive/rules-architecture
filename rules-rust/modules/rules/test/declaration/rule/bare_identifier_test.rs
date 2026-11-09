use crate::declaration::rule::BareIdentifier;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_persistent_code_fixtures_when_checked_then_match_exact_anchors() {
    assert_code_fixture(&BareIdentifier, "BareIdentifier", "bad");
    assert_code_fixture(&BareIdentifier, "BareIdentifier", "good");
}

#[test]
fn given_forbidden_code_when_checked_then_reports_exact_node_lines() {
    assert_code_findings(
        &BareIdentifier,
        "BareIdentifier",
        "mod model {\n    struct User { id: String, group_id: Option<u64> }\n    struct UserId { value: u64 }\n    enum Choice { User { user_id: uuid::Uuid } }\n}",
        &[2, 2, 3, 4],
    );
}

#[test]
fn given_prescribed_code_when_checked_then_passes() {
    assert_code_findings(
        &BareIdentifier,
        "BareIdentifier",
        "mod model {\n    struct UserId(u64);\n    struct User { id: UserId, value: String }\n    #[derive(serde::Serialize)] struct Wire { id: String }\n    #[derive(serde::Deserialize)] struct WireId { value: String }\n    enum StatusId { Empty }\n}\nmod file { struct Row { id: String } }",
        &[],
    );
}
