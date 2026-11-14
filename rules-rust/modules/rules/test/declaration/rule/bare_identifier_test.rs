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
fn given_single_field_serde_records_in_model_when_checked_then_the_format_key_keeps_its_named_field()
 {
    assert_code_findings(
        &BareIdentifier,
        "BareIdentifier",
        "mod model {\n#[derive(Debug, Clone, Serialize, Deserialize)] pub(crate) struct KeepInventoryConfiguration { is_enabled: bool }\n#[derive(serde::Deserialize)] pub(crate) struct PortalState { portals: Vec<Portal> }\n#[derive(Debug, serde::Serialize)] pub(crate) struct PluginTranslation { reload: Portal }\npub(crate) struct Portal;\n}",
        &[],
    );
}

#[test]
fn given_serde_derive_through_cfg_attr_when_checked_then_the_record_is_exempt_too() {
    assert_code_findings(
        &BareIdentifier,
        "BareIdentifier",
        "mod model {\n#[cfg_attr(feature = \"serde\", derive(Serialize, Deserialize))] struct Flags { enabled: bool }\n#[cfg_attr(feature = \"serde\", derive(serde::Deserialize))] struct State { portals: Vec<Flags> }\n}",
        &[],
    );
}

#[test]
fn given_serde_value_object_forms_when_checked_then_transparent_try_from_and_from_still_report() {
    assert_code_findings(
        &BareIdentifier,
        "BareIdentifier",
        "mod model {\n#[derive(Serialize, Deserialize)] #[serde(transparent)] struct Email { value: String }\n#[derive(Deserialize)] #[serde(try_from = \"String\")] struct Handle { value: String }\n#[derive(Deserialize)] #[serde(from = \"String\")] struct Tag { value: String }\n#[cfg_attr(feature = \"serde\", derive(Serialize), serde(transparent))] struct Title { value: String }\n#[derive(Serialize, Deserialize)] #[serde(default)] struct Record { value: String }\n}",
        &[2, 3, 4, 5],
    );
}

#[test]
fn given_single_field_structs_without_serde_when_checked_then_the_value_object_branch_still_reports()
 {
    assert_code_findings(
        &BareIdentifier,
        "BareIdentifier",
        "mod model {\n#[derive(Debug, Clone)] struct Money { cents: i64 }\n#[derive(Debug)] #[serde(default)] struct Basket { items: Vec<Money> }\n}",
        &[2, 3],
    );
}

#[test]
fn given_serde_structs_with_identifier_shapes_when_checked_then_only_the_single_property_branch_is_exempt()
 {
    assert_code_findings(
        &BareIdentifier,
        "BareIdentifier",
        "mod model {\n#[derive(Deserialize)] struct OrderId { value: String }\n#[derive(Serialize)] struct Wire { id: String }\n}",
        &[2, 3],
    );
}

const IDENTIFIER_MESSAGE: &str = "Use a single-field tuple identifier instead of bare strings, UUIDs or numbers (arch-domain.rs.md)";

const SINGLE_PROPERTY_MESSAGE: &str = "A model struct with one named field is a single-property value object: write it as a tuple struct (arch-domain.rs.md)";

fn list_messages(source: &str) -> Vec<String> {
    let context = crate::test::declaration::util::read_code_context(source);
    BareIdentifier
        .check(&context)
        .iter()
        .map(|finding| finding.finding.message.clone())
        .collect()
}

#[test]
fn given_single_field_struct_without_serde_in_model_when_checked_then_the_message_names_the_value_object()
 {
    assert_eq!(
        list_messages("mod model {\nstruct Basket { items: Vec<Item> }\nstruct Item;\n}"),
        [SINGLE_PROPERTY_MESSAGE]
    );
}

#[test]
fn given_braced_struct_named_id_when_checked_then_the_message_names_the_identifier() {
    assert_eq!(
        list_messages("mod model {\nstruct OrderId { value: String }\n}"),
        [IDENTIFIER_MESSAGE]
    );
}

#[test]
fn given_bare_id_field_when_checked_then_the_message_names_the_identifier() {
    assert_eq!(
        list_messages("mod model {\nstruct Order { customer_id: String, note: String }\n}"),
        [IDENTIFIER_MESSAGE]
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
