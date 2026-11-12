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
        "mod model {\nstruct Jail { jail_id: std::num::NonZeroU64, name: String }\nstruct Plot { id: std::borrow::Cow<'static, str>, name: String }\nstruct Room { id: std::rc::Rc<str>, name: String }\nstruct Gate { id: std::sync::Arc<String>, name: String }\nstruct Wing { id: Option<std::rc::Rc<String>>, name: String }\nstruct Hall { id: Option<Box<String>>, name: String }\nstruct Tag { id: Option<std::num::NonZero<u32>>, name: String }\nstruct JailId(u64); struct Fine { id: JailId, name: std::rc::Rc<str> }\n}\nmod file { struct Row { id: std::num::NonZeroU64, name: String } }",
        &[2, 3, 4, 5, 6, 7, 8],
    );
}

#[test]
fn given_plural_id_collection_fields_when_checked_then_requires_typed_ids() {
    assert_code_findings(
        &BareIdentifier,
        "BareIdentifier",
        "mod model {\nstruct Team { member_ids: Vec<uuid::Uuid>, name: String }\nstruct Squad { ids: std::collections::HashSet<String>, name: String }\nstruct TeamId(u64); struct Club { member_ids: Vec<TeamId>, kids: Vec<String> }\n}\nmod file { struct Row { member_ids: Vec<String>, name: String } }",
        &[2, 3],
    );
}

#[test]
fn given_identifier_type_aliases_in_model_when_checked_then_only_the_id_named_alias_reports() {
    assert_code_findings(
        &BareIdentifier,
        "BareIdentifier",
        "mod model {\npub(crate) type InvoiceId = uuid::Uuid;\npub(crate) type Outcome = Result<u32, String>;\npub(crate) struct CustomerId(uuid::Uuid);\n}",
        &[2],
    );
}

#[test]
fn given_single_field_structs_in_model_when_checked_then_the_identity_field_is_the_entity_mark() {
    assert_code_findings(
        &BareIdentifier,
        "BareIdentifier",
        "mod model {\nstruct PlayerId(u64);\nstruct Player { id: PlayerId }\nstruct Email { value: String }\nstruct Session { player_name: String }\n}",
        &[4, 5],
    );
}
