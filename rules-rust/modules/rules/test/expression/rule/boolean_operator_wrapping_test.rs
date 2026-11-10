use crate::expression::rule::BooleanOperatorWrapping;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_persistent_code_fixtures_when_checked_then_match_exact_anchors() {
    assert_code_fixture(&BooleanOperatorWrapping, "BooleanOperatorWrapping", "bad");
    assert_code_fixture(&BooleanOperatorWrapping, "BooleanOperatorWrapping", "good");
}

#[test]
fn given_forbidden_code_when_checked_then_reports_exact_node_lines() {
    assert_code_findings(
        &BooleanOperatorWrapping,
        "BooleanOperatorWrapping",
        "fn run() {\n    if ready\n        && current.is_valid() {}\n    while ready\n        || attempts > 3 {}\n    match value {\n        item if ready\n            && item.is_valid() => {},\n        _ => {}\n    }\n}",
        &[2, 4, 7],
    );
}

#[test]
fn given_prescribed_code_when_checked_then_passes() {
    assert_code_findings(
        &BooleanOperatorWrapping,
        "BooleanOperatorWrapping",
        "fn run() {\n    if ready\n        && !busy\n        && true {}\n    if ready && current.is_valid() {}\n    let condition = ready\n        && current.is_valid();\n}",
        &[],
    );
}

#[test]
fn given_tdd_boolean_operator_wrapping_multiline_helper_condition_when_checked_then_reports_missing_continuation()
 {
    assert_code_findings(
        &BooleanOperatorWrapping,
        "BooleanOperatorWrapping",
        r#"pub fn is_delivery_address_compatible_with_the_selected_distribution_region(
    address: &str,
    city: &str,
    postal_code: &str,
    region: &str,
) -> bool {
    !address.is_empty() && !city.is_empty() && !postal_code.is_empty() && !region.is_empty()
}

pub fn compute_delivery(address: &str, city: &str, postal_code: &str, region: &str) -> u8 {
    if is_delivery_address_compatible_with_the_selected_distribution_region(
        address,
        city,
        postal_code,
        region,
    ) {
        1
    } else {
        0
    }
}

pub fn compute_legacy_delivery(address: &str, city: &str, postal_code: &str, region: &str) -> u8 {
    if !address.trim().is_empty()
        && !city.trim().is_empty()
        && !postal_code.trim().is_empty()
        && !region.trim().is_empty()
    {
        1
    } else {
        0
    }
}

pub fn compute_delivery_with_named_condition(
    address: &str,
    city: &str,
    postal_code: &str,
    region: &str,
) -> u8 {
    let is_compatible = is_delivery_address_compatible_with_the_selected_distribution_region(
        address,
        city,
        postal_code,
        region,
    );
    if is_compatible { 1 } else { 0 }
}"#,
        &[11, 24],
    );
}
