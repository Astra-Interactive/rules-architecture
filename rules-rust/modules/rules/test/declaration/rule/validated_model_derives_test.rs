use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::declaration::rule::ValidatedModelDerives;
use crate::test::check::usecase::read_case_context;
use crate::test::declaration::util::{assert_code_findings, assert_code_fixture};
use crate::test::module::util::{assert_placement_findings, read_placement_context};

fn list_findings(context: &RuleContext) -> Vec<String> {
    let mut lines: Vec<String> = ValidatedModelDerives
        .check(context)
        .iter()
        .map(|finding| {
            format!(
                "{}:{} {}",
                finding.finding.location.path,
                finding.finding.location.line.expect("finding line"),
                finding.finding.subject
            )
        })
        .collect();
    lines.sort();
    lines
}

#[test]
fn given_validated_type_deriving_deserialize_without_try_from_when_checked_then_it_is_reported() {
    assert_code_findings(
        &ValidatedModelDerives,
        "ValidatedModelDerives",
        "mod model {\n#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)] struct Money(i64);\nimpl TryFrom<i64> for Money { type Error = (); fn try_from(cents: i64) -> Result<Self, ()> { Ok(Self(cents)) } }\n#[cfg_attr(feature = \"serde\", derive(Deserialize))] struct Level(u8);\nimpl TryFrom<u8> for Level { type Error = (); fn try_from(value: u8) -> Result<Self, ()> { Ok(Self(value)) } }\n#[derive(serde::Deserialize)] struct Rate { value: u8 }\nimpl std::convert::TryFrom<u8> for Rate { type Error = (); fn try_from(value: u8) -> Result<Self, ()> { Ok(Self { value }) } }\n}",
        &[2, 4, 6],
    );
}

#[test]
fn given_validated_type_deriving_deserialize_with_serde_try_from_when_checked_then_it_is_not_reported()
 {
    assert_code_findings(
        &ValidatedModelDerives,
        "ValidatedModelDerives",
        "mod model {\n#[derive(Debug, Deserialize)] #[serde(try_from = \"i64\")] struct Money(i64);\nimpl TryFrom<i64> for Money { type Error = (); fn try_from(cents: i64) -> Result<Self, ()> { Ok(Self(cents)) } }\n#[derive(Debug)] #[cfg_attr(feature = \"serde\", derive(Deserialize), serde(try_from = \"u8\"))] struct Level(u8);\nimpl TryFrom<u8> for Level { type Error = (); fn try_from(value: u8) -> Result<Self, ()> { Ok(Self(value)) } }\n#[derive(Debug, Deserialize)] #[cfg_attr(feature = \"serde\", serde(try_from = \"u8\"))] struct Rate(u8);\nimpl TryFrom<u8> for Rate { type Error = (); fn try_from(value: u8) -> Result<Self, ()> { Ok(Self(value)) } }\n}",
        &[],
    );
}

#[test]
fn given_derive_and_serde_attributes_under_nested_cfg_attr_when_checked_then_they_are_unwrapped() {
    assert_code_findings(
        &ValidatedModelDerives,
        "ValidatedModelDerives",
        "mod model {\n#[cfg_attr(feature = \"serde\", cfg_attr(not(test), derive(Deserialize)))] struct Money { cents: u64 }\nimpl TryFrom<u64> for Money { type Error = (); fn try_from(cents: u64) -> Result<Self, ()> { Ok(Self { cents }) } }\n#[cfg_attr(feature = \"serde\", cfg_attr(unix, cfg_attr(not(test), derive(Default))))] struct Level(u8);\nimpl TryFrom<u8> for Level { type Error = (); fn try_from(value: u8) -> Result<Self, ()> { Ok(Self(value)) } }\n#[derive(Deserialize)] #[cfg_attr(feature = \"serde\", cfg_attr(not(test), serde(try_from = \"u8\")))] struct Rate(u8);\nimpl TryFrom<u8> for Rate { type Error = (); fn try_from(value: u8) -> Result<Self, ()> { Ok(Self(value)) } }\n}",
        &[2, 4],
    );
}

#[test]
fn given_validated_type_deriving_default_when_checked_then_it_is_reported() {
    assert_code_findings(
        &ValidatedModelDerives,
        "ValidatedModelDerives",
        "mod model {\n#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)] struct Money(i64);\nimpl TryFrom<i64> for Money { type Error = (); fn try_from(cents: i64) -> Result<Self, ()> { Ok(Self(cents)) } }\n#[cfg_attr(test, derive(Default))] struct Level(u8);\nimpl TryFrom<u8> for Level { type Error = (); fn try_from(value: u8) -> Result<Self, ()> { Ok(Self(value)) } }\n}",
        &[2, 4],
    );
}

#[test]
fn given_plain_data_models_and_parts_structs_when_checked_then_derives_are_not_reported() {
    assert_code_findings(
        &ValidatedModelDerives,
        "ValidatedModelDerives",
        "mod model {\n#[derive(Debug, Deserialize, Default)] struct Settings { limit: u64 }\n#[derive(Debug, Deserialize, Default)] struct MoneyParts { pub(crate) cents: i64 }\n#[derive(Debug, Deserialize, Default)] struct Money { pub(crate) cents: i64 }\nimpl TryFrom<MoneyParts> for Money { type Error = (); fn try_from(parts: MoneyParts) -> Result<Self, ()> { Ok(Self { cents: parts.cents }) } }\n#[derive(Debug, Deserialize, Default)] struct Marker;\n}",
        &[],
    );
    let context = read_placement_context(&[
        (
            "src/lib.rs",
            "mod model { pub(crate) use money::Money; mod money; mod money_conversion; }",
        ),
        (
            "src/model/money.rs",
            "#[derive(Debug, Deserialize, Default)]\npub(crate) struct Money(i64);",
        ),
        (
            "src/model/money_conversion.rs",
            "use crate::model::Money;\nimpl TryFrom<i64> for Money { type Error = (); fn try_from(cents: i64) -> Result<Self, ()> { Ok(Self(cents)) } }",
        ),
    ]);
    assert_placement_findings(
        &ValidatedModelDerives,
        "ValidatedModelDerives",
        &context,
        &[],
    );
}

#[test]
fn given_persistent_derive_cases_when_checked_then_match_exact_node_anchors() {
    assert_code_fixture(&ValidatedModelDerives, "ValidatedModelDerives", "bad");
    assert_code_fixture(&ValidatedModelDerives, "ValidatedModelDerives", "good");
}

#[test]
fn given_invariant_derive_cases_when_checked_then_exact_wrong_sites_report_and_correct_types_pass()
{
    let context = read_case_context("ValidatedModelDerives", "fail-invariant-derives");
    assert_eq!(
        list_findings(&context),
        [
            "modules/sample/src/model/level.rs:2 case_sample::model::Level",
            "modules/sample/src/model/money.rs:2 case_sample::model::Money"
        ]
    );
    let context = read_case_context("ValidatedModelDerives", "pass-invariant-derives");
    assert!(ValidatedModelDerives.check(&context).is_empty());
}
