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
fn given_crate_visible_tuple_field_read_only_in_its_owner_file_when_checked_then_the_field_is_reported()
 {
    let root = "mod counter { pub(crate) use value::Counter; mod value; }\nmod report { pub(crate) use total::read_total; mod total; }\nfn run() -> u64 { report::read_total() }";
    let counter = "pub(crate) struct Counter(\n    pub(crate) u64,\n    pub(crate) u64,\n);\nimpl Counter {\n    pub(crate) fn new() -> Self { Self(0, 1) }\n    pub(crate) fn read(&self) -> u64 { self.0 }\n}";
    let total = "use crate::counter::Counter;\npub(crate) fn read_total() -> u64 { Counter::new().read() + Counter::new().1 }";
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
fn given_public_fields_of_crate_visible_structs_when_checked_then_reports_only_those_fields() {
    let source = "pub struct Open {\n    pub total: u64,\n}\npub(crate) struct Root {\n    pub value: u64,\n    pub(crate) step: u64,\n}\npub(crate) struct Pair(\n    pub u64,\n    pub(crate) u64,\n);";
    let context = read_published_placement_context(&[("src/lib.rs", source)]);
    assert_placement_findings(
        &VisibilityTier,
        "VisibilityTier",
        &context,
        &[("src/lib.rs", 5, "Node"), ("src/lib.rs", 9, "Node")],
    );
}

#[test]
fn given_public_fields_of_used_crate_visible_structs_in_an_unpublished_crate_when_checked_then_reports_only_those_fields()
 {
    let root = "mod shape {\n    pub(crate) use model::Pair;\n    pub(crate) use model::Root;\n    mod model;\n}\nmod report { pub(crate) use total::read_total; mod total; }\nfn run() -> u64 { report::read_total() }";
    let model = "pub(crate) struct Root {\n    pub value: u32,\n    pub(crate) step: u32,\n}\npub(crate) struct Pair(\n    pub u64,\n    pub(crate) u64,\n);";
    let total = "use crate::shape::Pair;\nuse crate::shape::Root;\npub(crate) fn read_total() -> u64 {\n    let root = Root { value: 1, step: 2 };\n    let pair = Pair(3, 4);\n    u64::from(root.step) + pair.1\n}";
    let context = read_private_placement_context(&[
        ("src/lib.rs", root),
        ("src/shape/model.rs", model),
        ("src/report/total.rs", total),
    ]);
    assert_placement_findings(
        &VisibilityTier,
        "VisibilityTier",
        &context,
        &[
            ("src/shape/model.rs", 2, "Node"),
            ("src/shape/model.rs", 6, "Node"),
        ],
    );
}

#[test]
fn given_crate_visible_tuple_fields_built_through_type_alias_or_renamed_import_when_checked_then_reports_only_the_unused_field()
 {
    let root = "mod shop {\n    pub(crate) use tally::AliasedTally;\n    pub(crate) use tally::RenamedTally;\n    pub(crate) use tally::UnusedTally;\n    mod tally;\n}\nmod report { pub(crate) use total::read_total; mod total; }\nfn run(unused: &shop::UnusedTally) -> usize { report::read_total(unused) }";
    let tally = "pub(crate) struct AliasedTally(pub(crate) u64);\npub(crate) struct RenamedTally(pub(crate) u64);\npub(crate) struct UnusedTally(pub(crate) u64);";
    let total = "use crate::shop::RenamedTally as Renamed;\nuse crate::shop::{AliasedTally, UnusedTally};\ntype Tally = AliasedTally;\npub(crate) fn read_total(unused: &UnusedTally) -> usize {\n    let _renamed = Renamed(2);\n    let _aliased = Tally(1);\n    0\n}";
    let context = read_private_placement_context(&[
        ("src/lib.rs", root),
        ("src/shop/tally.rs", tally),
        ("src/report/total.rs", total),
    ]);
    assert_placement_findings(
        &VisibilityTier,
        "VisibilityTier",
        &context,
        &[("src/shop/tally.rs", 3, "Node")],
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

#[test]
fn given_public_members_named_by_no_other_crate_when_checked_then_reports_each_member() {
    let billing = "pub struct Invoice {\n    pub total: u64,\n    pub paid: bool,\n}\nimpl Invoice {\n    pub fn describe(&self) -> u64 { self.total }\n    pub fn label(&self) -> bool { self.paid }\n}";
    let ledger = "fn show(invoice: &shop_billing_api::Invoice) -> bool {\n    invoice.paid && invoice.label()\n}";
    let context = crate::test::module::util::read_workspace_shape_context(&[
        ("shop_billing_api", "", billing),
        (
            "shop_ledger",
            "shop_billing_api={path=\"../shop_billing_api\"}",
            ledger,
        ),
    ]);
    assert_placement_findings(
        &VisibilityTier,
        "VisibilityTier",
        &context,
        &[
            ("shop_billing_api/src/lib.rs", 2, "Node"),
            ("shop_billing_api/src/lib.rs", 6, "Node"),
        ],
    );
}

#[test]
fn given_public_tuple_fields_no_other_crate_reads_when_checked_then_reports_each_struct_field() {
    let billing = "pub struct InvoiceTotal(pub u64);\npub struct InvoiceCount(pub u64);\npub struct InvoiceLabel(pub u64, pub bool);\npub struct InvoiceNote(\n    pub u64,\n    pub bool,\n);";
    let ledger = "use shop_billing_api::InvoiceCount;\nfn show(\n    total: &shop_billing_api::InvoiceTotal,\n    count: InvoiceCount,\n    label: &shop_billing_api::InvoiceLabel,\n    note: &shop_billing_api::InvoiceNote,\n) -> bool {\n    let InvoiceCount(value) = count;\n    value > 0 && note.1\n}";
    let context = crate::test::module::util::read_workspace_shape_context(&[
        ("shop_billing_api", "", billing),
        (
            "shop_ledger",
            "shop_billing_api={path=\"../shop_billing_api\"}",
            ledger,
        ),
    ]);
    assert_placement_findings(
        &VisibilityTier,
        "VisibilityTier",
        &context,
        &[
            ("shop_billing_api/src/lib.rs", 1, "Node"),
            ("shop_billing_api/src/lib.rs", 3, "Node"),
            ("shop_billing_api/src/lib.rs", 5, "Node"),
        ],
    );
}

#[test]
fn given_public_tuple_fields_built_through_type_alias_or_renamed_import_when_checked_then_reports_only_the_unused_field()
 {
    let billing = "pub struct AliasedTotal(pub u64);\npub struct RenamedCount(pub u64);\npub struct UnusedLabel(pub u64);";
    let ledger = "use shop_billing_api::RenamedCount as Renamed;\ntype Total = shop_billing_api::AliasedTotal;\nfn show(label: &shop_billing_api::UnusedLabel, count: &shop_billing_api::RenamedCount) -> Total {\n    let renamed = Renamed(2);\n    Total(1)\n}";
    let context = crate::test::module::util::read_workspace_shape_context(&[
        ("shop_billing_api", "", billing),
        (
            "shop_ledger",
            "shop_billing_api={path=\"../shop_billing_api\"}",
            ledger,
        ),
    ]);
    assert_placement_findings(
        &VisibilityTier,
        "VisibilityTier",
        &context,
        &[("shop_billing_api/src/lib.rs", 3, "Node")],
    );
}

#[test]
fn given_workspace_item_named_only_in_a_select_handler_when_checked_then_the_handler_counts_as_its_user()
 {
    let worker = "async fn watch(mut tick: tokio::sync::mpsc::Receiver<u32>) {\n    tokio::select! {\n        value = tick.recv() => {\n            drop(value);\n            drop(crate::ticker::Ticker);\n        }\n    }\n}";
    let context = read_private_placement_context(&[
        ("src/lib.rs", "mod lonely;\nmod ticker;\nmod worker;"),
        ("src/lonely.rs", "pub(crate) struct Lonely;"),
        ("src/ticker.rs", "pub(crate) struct Ticker;"),
        ("src/worker.rs", worker),
    ]);
    assert_placement_findings(
        &VisibilityTier,
        "VisibilityTier",
        &context,
        &[("src/lonely.rs", 1, "Node")],
    );
}

#[test]
fn given_select_branch_binding_a_module_name_when_another_branch_uses_it_then_the_item_is_used() {
    let root = "mod lonely;\nmod ticker;\nasync fn watch(mut inbox: tokio::sync::mpsc::Receiver<u32>, mut stop: tokio::sync::mpsc::Receiver<u32>) {\n    tokio::select! {\n        ticker = inbox.recv() => {\n            drop(ticker);\n        }\n        stopped = stop.recv() => {\n            drop(stopped);\n            drop(ticker::Ticker);\n        }\n    }\n}";
    let context = read_private_placement_context(&[
        ("src/lib.rs", root),
        ("src/lonely.rs", "pub(crate) struct Lonely;"),
        ("src/ticker.rs", "pub(crate) struct Ticker;"),
    ]);
    assert_placement_findings(
        &VisibilityTier,
        "VisibilityTier",
        &context,
        &[("src/lonely.rs", 1, "Node")],
    );
}

#[test]
fn given_select_pattern_binding_a_module_name_when_its_own_future_uses_it_then_the_item_is_used() {
    let root = "mod lonely;\nmod ticker;\nasync fn watch() {\n    tokio::select! {\n        ticker = ticker::next() => {}\n    }\n}";
    let context = read_private_placement_context(&[
        ("src/lib.rs", root),
        ("src/lonely.rs", "pub(crate) struct Lonely;"),
        ("src/ticker.rs", "pub(crate) async fn next() {}"),
    ]);
    assert_placement_findings(
        &VisibilityTier,
        "VisibilityTier",
        &context,
        &[("src/lonely.rs", 1, "Node")],
    );
}

#[test]
fn given_super_and_self_visibility_when_checked_then_reports_item_field_and_method() {
    let context = read_private_placement_context(&[
        ("src/lib.rs", "mod table;"),
        (
            "src/table.rs",
            "pub(super) struct Row {\n    pub(super) id: u32,\n}\nimpl Row { pub(self) fn open(&self) {} }",
        ),
    ]);
    assert_placement_findings(
        &VisibilityTier,
        "VisibilityTier",
        &context,
        &[
            ("src/table.rs", 1, "Node"),
            ("src/table.rs", 2, "Node"),
            ("src/table.rs", 4, "Node"),
        ],
    );
}

#[test]
fn given_self_and_path_visibility_beside_crate_visibility_when_checked_then_reports_only_those() {
    let root = "mod table;\nmod keep;\nfn run() -> u32 { keep::Keep::new().read() + keep::Keep::new().step }";
    let keep = "pub(crate) struct Keep {\n    pub(crate) step: u32,\n}\nimpl Keep {\n    pub(crate) fn new() -> Self { Self { step: 1 } }\n    pub(crate) fn read(&self) -> u32 { self.step }\n}";
    let table = "pub(self) struct Hidden;\nstruct Zone {\n    pub(in crate::table) code: u32,\n}\nimpl Zone {\n    pub(in crate::table) fn close(&self) {}\n}";
    let context = read_private_placement_context(&[
        ("src/lib.rs", root),
        ("src/keep.rs", keep),
        ("src/table.rs", table),
    ]);
    assert_placement_findings(
        &VisibilityTier,
        "VisibilityTier",
        &context,
        &[
            ("src/table.rs", 1, "Node"),
            ("src/table.rs", 3, "Node"),
            ("src/table.rs", 6, "Node"),
        ],
    );
}

#[test]
fn given_published_billing_crate_when_public_members_are_unused_then_reports_no_member() {
    let billing = "pub struct Invoice {\n    pub total: u64,\n    pub paid: bool,\n}\nimpl Invoice {\n    pub fn describe(&self) -> u64 { self.total }\n    pub fn label(&self) -> bool { self.paid }\n}";
    let ledger = "fn show(invoice: &shop_billing_api::Invoice) -> bool {\n    invoice.paid && invoice.label()\n}";
    let mut context = crate::test::module::util::read_workspace_shape_context(&[
        ("shop_billing_api", "", billing),
        (
            "shop_ledger",
            "shop_billing_api={path=\"../shop_billing_api\"}",
            ledger,
        ),
    ]);
    for member in &mut context.workspace.members {
        if member.name == "shop_billing_api" {
            member.manifest = crate::workspace::util::parse_toml_file(
                "Cargo.toml",
                "[package]\nname=\"shop_billing_api\"\nversion=\"0.1.0\"\n",
            )
            .expect("published manifest");
        }
    }
    assert_placement_findings(&VisibilityTier, "VisibilityTier", &context, &[]);
}
