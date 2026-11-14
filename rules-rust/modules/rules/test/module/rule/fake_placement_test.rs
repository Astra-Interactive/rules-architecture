use crate::check::api::Rule;
use crate::module::rule::FakePlacement;
use crate::source::model::SourceTree;
use crate::test::check::usecase::read_case_context;
use crate::test::declaration::util::assert_code_fixture;
use crate::test::module::util::{
    assert_placement_findings, read_placement_context, read_workspace_shape_context,
};
use crate::workspace::util::parse_toml_file;

#[test]
fn given_fixture_feature_pair_and_duplicate_fakes_when_checked_then_reports_exact_tree_declarations()
 {
    let source = "mod account { mod api { trait Reader {} } }\n#[cfg(test)] mod test { mod account { mod fake { pub(crate) struct FakeReader; impl crate::account::api::Reader for FakeReader {} } } }\n#[cfg(any(test, feature = \"fake\"))] mod fixture { mod account { mod fake { pub struct FakeReader; impl crate::account::api::Reader for FakeReader {} } } }";
    let mut context = read_placement_context(&[("src/lib.rs", source)]);
    let path = context.workspace.members[0].manifest.path.clone();
    context.workspace.members[0].manifest =
        parse_toml_file(&path, "[features]\nfake = []\n").expect("fake feature");
    assert_placement_findings(
        &FakePlacement,
        "FakePlacement",
        &context,
        &[("src/lib.rs", 2, "Node"), ("src/lib.rs", 3, "Node")],
    );
}

#[test]
fn given_fixture_without_feature_and_feature_without_fixture_when_checked_then_reports_crate_subject()
 {
    let mut context = read_placement_context(&[(
        "src/lib.rs",
        "#[cfg(any(test, feature = \"fake\"))] mod fixture {}",
    )]);
    let path = context.workspace.members[0].manifest.path.clone();
    assert_placement_findings(
        &FakePlacement,
        "FakePlacement",
        &context,
        &[(path.as_str(), 0, "Crate")],
    );
    context.workspace.members[0].manifest =
        parse_toml_file(&path, "[features]\nfake = []\n").expect("fake feature");
    assert_placement_findings(&FakePlacement, "FakePlacement", &context, &[]);
    let mut no_fixture = read_placement_context(&[("src/lib.rs", "")]);
    no_fixture.workspace.members[0].manifest = context.workspace.members[0].manifest.clone();
    assert_placement_findings(
        &FakePlacement,
        "FakePlacement",
        &no_fixture,
        &[(path.as_str(), 2, "Crate")],
    );
}

#[test]
fn given_default_feature_enabling_fake_when_checked_then_reports_the_default_key() {
    let mut context = read_placement_context(&[(
        "src/lib.rs",
        "#[cfg(any(test, feature = \"fake\"))] mod fixture {}",
    )]);
    let path = context.workspace.members[0].manifest.path.clone();
    let manifest = "[features]\ndefault = [\"fake\"]\nfake = []\n";
    context.workspace.members[0].manifest =
        parse_toml_file(&path, manifest).expect("default feature enabling fake");
    assert_placement_findings(
        &FakePlacement,
        "FakePlacement",
        &context,
        &[(path.as_str(), 2, "Crate")],
    );
    context.workspace.members[0].manifest =
        parse_toml_file(&path, "[features]\ndefault = []\nfake = []\n").expect("empty default");
    assert_placement_findings(&FakePlacement, "FakePlacement", &context, &[]);
}

#[test]
fn given_default_feature_reaching_fake_through_another_feature_when_checked_then_reports_the_default_key()
 {
    let mut context = read_placement_context(&[(
        "src/lib.rs",
        "#[cfg(any(test, feature = \"fake\"))] mod fixture {}",
    )]);
    let path = context.workspace.members[0].manifest.path.clone();
    let manifest = "[features]\ndefault = [\"testing\"]\ntesting = [\"fake\"]\nfake = []\n";
    context.workspace.members[0].manifest =
        parse_toml_file(&path, manifest).expect("default feature reaching fake");
    assert_placement_findings(
        &FakePlacement,
        "FakePlacement",
        &context,
        &[(path.as_str(), 2, "Crate")],
    );
    let manifest = "[features]\ndefault = [\"testing\"]\ntesting = [\"loop\"]\nloop = [\"testing\"]\nfake = []\n";
    context.workspace.members[0].manifest =
        parse_toml_file(&path, manifest).expect("default feature never reaching fake");
    assert_placement_findings(&FakePlacement, "FakePlacement", &context, &[]);
}

#[test]
fn given_fake_outside_its_trait_concept_or_wrong_visibility_when_checked_then_reports_the_type() {
    for visibility in ["", "pub"] {
        let source = format!(
            "mod account {{ mod api {{ trait Reader {{}} }} }}\n#[cfg(test)] mod test {{ mod account {{ mod fake {{ {visibility} struct FakeReader; impl crate::account::api::Reader for FakeReader {{}} }} }} }}"
        );
        let context = read_placement_context(&[("src/lib.rs", &source)]);
        assert_placement_findings(
            &FakePlacement,
            "FakePlacement",
            &context,
            &[("src/lib.rs", 2, "Node")],
        );
    }
    let context = read_placement_context(&[(
        "src/lib.rs",
        "mod account { mod api { trait Reader {} pub use Reader as Contract; } }\n#[cfg(test)] mod test { mod billing { mod fake { pub(crate) struct FakeReader; impl crate::account::api::Contract for FakeReader {} } } }",
    )]);
    assert_placement_findings(
        &FakePlacement,
        "FakePlacement",
        &context,
        &[("src/lib.rs", 2, "Node")],
    );
}

#[test]
fn given_production_fake_kind_and_nonempty_feature_when_checked_then_reports_module_and_feature() {
    let mut context = read_placement_context(&[(
        "src/lib.rs",
        "mod fake {}\n#[cfg(any(test, feature = \"fake\"))] mod fixture {}",
    )]);
    let path = context.workspace.members[0].manifest.path.clone();
    context.workspace.members[0].manifest =
        parse_toml_file(&path, "[features]\nfake = [\"support\"]\n").expect("feature contents");
    assert_placement_findings(
        &FakePlacement,
        "FakePlacement",
        &context,
        &[("src/lib.rs", 1, "Module"), (path.as_str(), 2, "Crate")],
    );
}

#[test]
fn given_shared_fake_in_a_crate_that_does_not_own_the_trait_when_checked_then_only_that_fake_reports()
 {
    let context = read_workspace_shape_context(&[
        (
            "shop_account_api",
            "\n[features]\nfake = []\n",
            "pub mod account { pub mod api { pub trait Reader { fn read(&self) -> u64; } } }\n#[cfg(any(test, feature = \"fake\"))]\npub mod fixture {\n    pub mod account {\n        pub mod fake {\n            pub struct FakeOwnedReader;\n            impl crate::account::api::Reader for FakeOwnedReader {\n                fn read(&self) -> u64 { 0 }\n            }\n        }\n    }\n}",
        ),
        (
            "shop_billing_api",
            "shop_account_api={path=\"../shop_account_api\"}\n[features]\nfake = []\n",
            "#[cfg(any(test, feature = \"fake\"))]\npub mod fixture {\n    pub mod account {\n        pub mod fake {\n            pub struct FakeReader;\n            impl shop_account_api::account::api::Reader for FakeReader {\n                fn read(&self) -> u64 { 0 }\n            }\n        }\n    }\n}",
        ),
    ]);
    let findings = FakePlacement.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!("FakePlacement", findings[0].finding.rule_id.0);
    assert_eq!(Some(5), findings[0].finding.location.line);
    assert!(
        findings[0]
            .finding
            .location
            .path
            .starts_with("shop_billing_api/")
    );
}

const DOUBLE_WITHOUT_FAKE_NAME: &str =
    "pub(crate) struct RecordingReader; impl crate::account::api::Reader for RecordingReader {}";

#[test]
fn given_file_of_a_test_fake_block_without_a_fake_type_when_checked_then_the_file_is_reported() {
    let context = read_placement_context(&[
        (
            "src/lib.rs",
            "mod account { mod api { pub(crate) trait Reader {} } }\n#[cfg(test)] mod test { mod account { mod fake { mod recording_reader; } } }",
        ),
        (
            "src/test/account/fake/recording_reader.rs",
            DOUBLE_WITHOUT_FAKE_NAME,
        ),
    ]);
    assert_placement_findings(
        &FakePlacement,
        "FakePlacement",
        &context,
        &[("src/lib.rs", 2, "Module")],
    );
}

#[test]
fn given_file_of_a_fixture_fake_block_without_a_fake_type_when_checked_then_the_file_is_reported() {
    let mut context = read_placement_context(&[
        (
            "src/lib.rs",
            "pub mod account { pub mod api { pub trait Reader {} } }\n#[cfg(any(test, feature = \"fake\"))] pub mod fixture { pub mod account { pub mod fake { mod recording_reader; } } }",
        ),
        (
            "src/fixture/account/fake/recording_reader.rs",
            "pub struct RecordingReader; impl crate::account::api::Reader for RecordingReader {}",
        ),
    ]);
    let path = context.workspace.members[0].manifest.path.clone();
    context.workspace.members[0].manifest =
        parse_toml_file(&path, "[features]\nfake = []\n").expect("fake feature");
    assert_placement_findings(
        &FakePlacement,
        "FakePlacement",
        &context,
        &[("src/lib.rs", 2, "Module")],
    );
}

#[test]
fn given_fake_file_with_a_plain_data_struct_only_its_fake_uses_when_checked_then_nothing_is_reported()
 {
    let context = read_placement_context(&[
        (
            "src/lib.rs",
            "mod account { mod api { pub(crate) trait Reader {} } }\n#[cfg(test)] mod test { mod account { mod fake { mod fake_reader; } } }",
        ),
        (
            "src/test/account/fake/fake_reader.rs",
            "pub(crate) struct FakeReader { calls: Vec<ReadCall> }\nstruct ReadCall;\nimpl crate::account::api::Reader for FakeReader {}",
        ),
    ]);
    assert_placement_findings(&FakePlacement, "FakePlacement", &context, &[]);
}

#[test]
fn given_double_named_otherwise_in_a_memory_block_when_checked_then_fake_placement_is_silent() {
    let context = read_placement_context(&[
        (
            "src/lib.rs",
            "mod account { mod api { pub(crate) trait Reader {} } }\n#[cfg(test)] mod test { mod account { mod memory { mod recording_reader; } } }",
        ),
        (
            "src/test/account/memory/recording_reader.rs",
            DOUBLE_WITHOUT_FAKE_NAME,
        ),
    ]);
    assert_placement_findings(&FakePlacement, "FakePlacement", &context, &[]);
}

#[test]
fn given_phase_one_fake_crate_cases_when_checked_then_preserves_the_crate_rule() {
    let bad = read_case_context("FakePlacement", "fail-crate");
    let good = read_case_context("FakePlacement", "pass-feature");
    assert_eq!(1, FakePlacement.check(&bad).len());
    assert!(FakePlacement.check(&good).is_empty());
}

#[test]
fn given_persistent_placement_cases_when_checked_then_match_exact_node_anchors() {
    assert_code_fixture(&FakePlacement, "FakePlacement", "bad");
    assert_code_fixture(&FakePlacement, "FakePlacement", "good");
}

#[test]
fn given_test_fake_of_another_crates_trait_when_checked_then_reports_it() {
    let context = read_workspace_shape_context(&[
        (
            "shop_billing_api",
            "",
            "pub mod invoice { pub mod api { pub trait InvoiceRepository {} } }",
        ),
        (
            "shop_ledger",
            "shop_billing_api={path=\"../shop_billing_api\"}",
            "mod ledger { pub(crate) mod api { pub(crate) trait LedgerReader {} } }\n#[cfg(test)] mod test {\nmod invoice { mod fake { pub(crate) struct FakeInvoiceRepository; impl shop_billing_api::invoice::api::InvoiceRepository for FakeInvoiceRepository {} } }\nmod ledger { mod fake { pub(crate) struct FakeLedgerReader; impl crate::ledger::api::LedgerReader for FakeLedgerReader {} } }\n}",
        ),
    ]);
    assert_placement_findings(
        &FakePlacement,
        "FakePlacement",
        &context,
        &[("shop_ledger/src/lib.rs", 3, "Node")],
    );
}

const BILLING_WITH_FIXTURE: &str = "pub mod invoice { pub mod api { pub trait InvoiceRepository {} } }\n#[cfg(any(test, feature = \"fake\"))]\npub mod fixture {\n    pub mod invoice {\n        pub mod fake {\n            pub struct FakeInvoiceRepository;\n            impl crate::invoice::api::InvoiceRepository for FakeInvoiceRepository {}\n        }\n    }\n}";

const BILLING_DEPENDENCY: &str = "shop_billing_api={path=\"../shop_billing_api\"}\n";

fn read_ledger_context(ledger: &str, features: &str) -> crate::check::model::RuleContext {
    read_workspace_shape_context(&[
        (
            "shop_billing_api",
            "\n[features]\nfake = []\n",
            BILLING_WITH_FIXTURE,
        ),
        (
            "shop_ledger",
            &format!("{BILLING_DEPENDENCY}{features}"),
            ledger,
        ),
    ])
}

fn list_fixture_path_findings(context: &crate::check::model::RuleContext) -> Vec<String> {
    let mut actual: Vec<String> = FakePlacement
        .check(context)
        .iter()
        .map(|finding| {
            format!(
                "{}:{} {}",
                finding.finding.location.path,
                finding.finding.location.line.unwrap_or(0),
                finding.finding.subject
            )
        })
        .collect();
    actual.sort();
    actual
}

#[test]
fn given_production_paths_into_the_fixture_tree_of_another_crate_when_checked_then_each_path_reports()
 {
    let context = read_ledger_context(
        "pub mod ledger {\n    pub mod util {\n        use shop_billing_api::fixture::invoice::fake::FakeInvoiceRepository;\n        pub fn first() { let repository = FakeInvoiceRepository; }\n        pub fn second() { let repository = shop_billing_api::fixture::invoice::fake::FakeInvoiceRepository; }\n        pub fn third(repository: &dyn shop_billing_api::invoice::api::InvoiceRepository) {}\n    }\n}",
        "",
    );
    let subject = "shop_billing_api::fixture::invoice::fake::FakeInvoiceRepository";
    assert_eq!(
        list_fixture_path_findings(&context),
        [
            format!("shop_ledger/src/lib.rs:3 {subject}"),
            format!("shop_ledger/src/lib.rs:4 {subject}"),
            format!("shop_ledger/src/lib.rs:5 {subject}"),
        ]
    );
}

#[test]
fn given_a_path_into_a_fixture_module_behind_a_test_or_fake_cfg_when_indexed_then_it_survives_as_a_production_use()
 {
    let context = read_ledger_context(
        "pub mod ledger {\n    pub fn first() { let repository = shop_billing_api::fixture::invoice::fake::FakeInvoiceRepository; }\n}",
        "",
    );
    let uses_of_fixture = context.name_index().uses.iter().filter(|usage| {
        let target = &context.indexes[usage.target_index];
        target.crate_name == "shop_billing_api"
            && target.modules[target.items[usage.target_item].module].tree == SourceTree::Fixture
            && context.indexes[usage.source_index].crate_name == "shop_ledger"
    });
    assert_eq!(
        1,
        uses_of_fixture.filter(|usage| !usage.is_test).count(),
        "the fixture cfg (`any(test, feature = \"fake\")`) is satisfiable without `test`"
    );
}

#[test]
fn given_paths_into_the_fixture_tree_from_tests_fixtures_or_the_own_crate_when_checked_then_none_reports()
 {
    let context = read_ledger_context(
        "pub mod ledger {\n    pub fn item() {}\n}\n#[cfg(test)]\nmod test {\n    mod ledger {\n        use shop_billing_api::fixture::invoice::fake::FakeInvoiceRepository;\n        fn first() { let repository = FakeInvoiceRepository; }\n    }\n}\n#[cfg(any(test, feature = \"fake\"))]\npub mod fixture {\n    pub mod ledger {\n        pub mod fake {\n            use shop_billing_api::fixture::invoice::fake::FakeInvoiceRepository;\n            pub fn wrap() { let repository = FakeInvoiceRepository; }\n        }\n    }\n}",
        "[features]\nfake = []\n",
    );
    assert_eq!(list_fixture_path_findings(&context), Vec::<String>::new());
    let own = read_workspace_shape_context(&[(
        "shop_billing_api",
        "\n[features]\nfake = []\n",
        "pub mod invoice { pub mod api { pub trait InvoiceRepository {} } pub mod util { pub fn first() { let repository = crate::fixture::invoice::fake::FakeInvoiceRepository; } } }\n#[cfg(any(test, feature = \"fake\"))]\npub mod fixture {\n    pub mod invoice {\n        pub mod fake {\n            pub struct FakeInvoiceRepository;\n            impl crate::invoice::api::InvoiceRepository for FakeInvoiceRepository {}\n        }\n    }\n}",
    )]);
    assert_eq!(list_fixture_path_findings(&own), Vec::<String>::new());
}

#[test]
fn given_fixture_path_cases_when_checked_then_production_paths_report_and_test_fixture_and_integration_paths_pass()
 {
    let bad = read_case_context("FakePlacement", "fail-production-fixture-path");
    let subject = "case_billing_api::fixture::invoice::fake::FakeInvoiceRepository";
    let path = "modules/ledger/src/ledger/usecase/count_ledger_use_case.rs";
    assert_eq!(
        list_fixture_path_findings(&bad),
        [
            format!("{path}:1 {subject}"),
            format!("{path}:15 {subject}")
        ]
    );
    let good = read_case_context("FakePlacement", "pass-test-fixture-paths");
    assert!(
        good.crates.iter().any(|sources| sources.target == "it"),
        "the published crate's tests/it is read"
    );
    assert_eq!(list_fixture_path_findings(&good), Vec::<String>::new());
}
