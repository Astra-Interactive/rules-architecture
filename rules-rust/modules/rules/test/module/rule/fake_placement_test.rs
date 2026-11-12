use crate::check::api::Rule;
use crate::module::rule::FakePlacement;
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
