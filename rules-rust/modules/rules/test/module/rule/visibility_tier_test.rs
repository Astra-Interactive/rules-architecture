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

fn read_private_placement_context(files: &[(&str, &str)]) -> crate::check::model::RuleContext {
    let mut context = read_placement_context(files);
    let name = context.workspace.members[0].name.clone();
    context.workspace.members[0].manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        &format!("[package]\nname={name:?}\npublish=false\n"),
    )
    .unwrap();
    context
}

#[test]
fn given_crate_visible_field_read_only_in_its_owner_file_when_checked_then_the_field_is_reported() {
    let root = "mod counter { pub(crate) use value::Counter; mod value; }\nmod report { pub(crate) use total::read_total; mod total; }\nfn run() -> u64 { report::read_total() }";
    let counter = "pub(crate) struct Counter {\n    pub(crate) value: u64,\n    pub(crate) step: u64,\n}\nimpl Counter {\n    pub(crate) fn new() -> Self { Self { value: 0, step: 1 } }\n    pub(crate) fn read(&self) -> u64 { self.value }\n}";
    let total = "use crate::counter::Counter;\npub(crate) fn read_total() -> u64 { Counter::new().read() + Counter::new().step }";
    let context = read_private_placement_context(&[
        ("src/lib.rs", root),
        ("src/counter/value.rs", counter),
        ("src/report/total.rs", total),
    ]);
    assert_placement_findings(
        &VisibilityTier,
        "VisibilityTier",
        &context,
        &[("src/counter/value.rs", 2, "Node")],
    );
}

#[test]
fn given_path_restricted_visibility_on_item_and_re_export_when_checked_then_the_item_is_reported() {
    let root = "mod invoice {\n    mod file {\n        pub(in crate::invoice) use file_invoice_repository::FileInvoiceRepository;\n        mod file_invoice_repository;\n    }\n    mod di {\n        pub(crate) use invoice_module::InvoiceModule;\n        mod invoice_module;\n    }\n}\nfn run(module: invoice::di::InvoiceModule) {}";
    let repository =
        "pub(in crate::invoice) struct FileInvoiceRepository {\n    folder: std::path::PathBuf,\n}";
    let module = "use crate::invoice::file::FileInvoiceRepository;\npub(crate) struct InvoiceModule {\n    repository: FileInvoiceRepository,\n}";
    let context = read_private_placement_context(&[
        ("src/lib.rs", root),
        ("src/invoice/file/file_invoice_repository.rs", repository),
        ("src/invoice/di/invoice_module.rs", module),
    ]);
    assert_placement_findings(
        &VisibilityTier,
        "VisibilityTier",
        &context,
        &[("src/invoice/file/file_invoice_repository.rs", 1, "Node")],
    );
}

#[test]
fn given_persistent_placement_cases_when_checked_then_match_exact_node_anchors() {
    assert_code_fixture(&VisibilityTier, "VisibilityTier", "bad");
    assert_code_fixture(&VisibilityTier, "VisibilityTier", "good");
}
