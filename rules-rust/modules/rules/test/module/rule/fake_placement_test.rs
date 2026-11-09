use crate::check::api::Rule;
use crate::module::rule::FakePlacement;
use crate::test::check::usecase::read_case_context;
use crate::test::declaration::util::assert_code_fixture;
use crate::test::module::util::{assert_placement_findings, read_placement_context};
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
