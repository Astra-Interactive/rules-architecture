use crate::check::api::Rule;
use crate::declaration::rule::BareIdentifier;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_custom_scalar_names_and_declared_wire_role_when_checked_then_preserves_their_identity() {
    assert_code_findings(
        &BareIdentifier,
        "BareIdentifier",
        "mod model { struct String; struct Uuid; struct Option<T>(T); struct User { id: String, other_id: Uuid, key_id: Option<u64> } }",
        &[],
    );
    let mut context = crate::test::declaration::util::read_code_context(
        "mod model { #[derive(serde::Serialize)] struct Wire { id: String, name: String } }",
    );
    context.project_config.crates[0].role = Some(crate::config::model::Role::Contract);
    assert!(BareIdentifier.check(&context).is_empty());
}

#[test]
fn given_serialized_domain_identifiers_and_aliases_when_checked_then_requires_typed_ids() {
    assert_code_findings(
        &BareIdentifier,
        "BareIdentifier",
        "type RawId = String; mod model {\n#[derive(serde::Serialize)] struct User { id: String, name: String }\n#[derive(serde::Deserialize)] struct Group { group_id: crate::RawId, name: String }\nstruct UserId(u64); #[derive(serde::Serialize)] struct Typed { id: UserId, name: String }\n}\nmod file { #[derive(serde::Serialize)] struct Wire { id: String } }",
        &[2, 3],
    );
}

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
        "mod model {\n    struct UserId(u64);\n    struct User { id: UserId, value: String }\n    #[derive(serde::Serialize)] struct Wire { id: UserId, value: String }\n    #[derive(serde::Deserialize)] struct WireId(String);\n    enum StatusId { Empty }\n}\nmod file { struct Row { id: String } }",
        &[],
    );
}

#[test]
fn given_standard_library_string_and_integer_wrappers_when_checked_then_requires_typed_ids() {
    assert_code_findings(
        &BareIdentifier,
        "BareIdentifier",
        "mod model {\nstruct Jail { jail_id: std::num::NonZeroU64 }\nstruct Plot { id: std::borrow::Cow<'static, str> }\nstruct Room { id: std::rc::Rc<str> }\nstruct Gate { id: std::sync::Arc<String> }\nstruct Wing { id: Option<std::rc::Rc<String>> }\nstruct Hall { id: Option<Box<String>> }\nstruct Tag { id: Option<std::num::NonZero<u32>> }\nstruct JailId(u64); struct Fine { id: JailId, name: std::rc::Rc<str> }\n}\nmod file { struct Row { id: std::num::NonZeroU64 } }",
        &[2, 3, 4, 5, 6, 7, 8],
    );
}
