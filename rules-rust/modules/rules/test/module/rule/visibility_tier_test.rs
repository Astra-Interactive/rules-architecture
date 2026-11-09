use crate::module::rule::VisibilityTier;
use crate::test::declaration::util::assert_code_fixture;
use crate::test::module::util::{assert_placement_findings, read_placement_context};

fn read_published_placement_context(files: &[(&str, &str)]) -> crate::check::model::RuleContext {
    let mut context = read_placement_context(files);
    context.workspace.members[0].manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[package]\nname=\"case-sample\"\nversion=\"0.1.0\"\n",
    )
    .unwrap();
    context
}

#[test]
fn given_crate_private_reexport_and_cfg_public_reexport_when_checked_then_tracks_actual_reachability()
 {
    let root = "pub mod model { pub(crate) use hidden::Value; mod hidden; }";
    let context = read_published_placement_context(&[
        ("src/lib.rs", root),
        ("src/model/hidden.rs", "pub struct Value { pub value: u64 }"),
    ]);
    assert_placement_findings(
        &VisibilityTier,
        "VisibilityTier",
        &context,
        &[("src/model/hidden.rs", 1, "Node")],
    );
    let root = "pub mod model { #[cfg(feature = \"open\")] pub use hidden::Value; mod hidden; }";
    let types = "#[cfg(feature = \"open\")] pub struct Value { pub value: u64 }\n#[cfg(not(feature = \"open\"))] pub struct Value { pub value: u64 }";
    let context =
        read_published_placement_context(&[("src/lib.rs", root), ("src/model/hidden.rs", types)]);
    assert_placement_findings(
        &VisibilityTier,
        "VisibilityTier",
        &context,
        &[("src/model/hidden.rs", 2, "Node")],
    );
}

#[test]
fn given_public_alias_through_private_parent_when_checked_then_the_struct_is_reachable() {
    let root = "pub use sample::model::Value as PublicValue; mod sample { pub mod model { pub use value::Value; mod value; } }";
    let context = read_published_placement_context(&[
        ("src/lib.rs", root),
        (
            "src/sample/model/value.rs",
            "pub struct Value { pub value: u64 }",
        ),
    ]);
    assert_placement_findings(&VisibilityTier, "VisibilityTier", &context, &[]);
    let context =
        read_published_placement_context(&[("src/lib.rs", "union Bits { pub value: u64 }")]);
    assert_placement_findings(
        &VisibilityTier,
        "VisibilityTier",
        &context,
        &[("src/lib.rs", 1, "Node")],
    );
}

#[test]
fn given_public_type_alias_or_signature_when_checked_then_fields_of_the_exposed_type_are_reachable()
{
    for exposed in [
        "pub type PublicValue = hidden::Value;",
        "pub fn load() -> hidden::Value { loop {} }",
    ] {
        let root = format!("{exposed} mod hidden;");
        let context = read_published_placement_context(&[
            ("src/lib.rs", &root),
            ("src/hidden.rs", "pub struct Value { pub value: u64 }"),
        ]);
        assert_placement_findings(&VisibilityTier, "VisibilityTier", &context, &[]);
    }
    let root = "mod hidden; pub fn accept<Value>(value: Value) {}";
    let context = read_published_placement_context(&[
        ("src/lib.rs", root),
        ("src/hidden.rs", "pub struct Value { pub value: u64 }"),
    ]);
    assert_placement_findings(
        &VisibilityTier,
        "VisibilityTier",
        &context,
        &[("src/hidden.rs", 1, "Node")],
    );
}

#[test]
fn given_persistent_placement_cases_when_checked_then_match_exact_node_anchors() {
    assert_code_fixture(&VisibilityTier, "VisibilityTier", "bad");
    assert_code_fixture(&VisibilityTier, "VisibilityTier", "good");
}
