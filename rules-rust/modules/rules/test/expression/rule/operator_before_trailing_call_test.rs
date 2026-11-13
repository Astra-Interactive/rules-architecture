use crate::expression::rule::OperatorBeforeTrailingCall;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_persistent_code_fixtures_when_checked_then_match_exact_anchors() {
    assert_code_fixture(
        &OperatorBeforeTrailingCall,
        "OperatorBeforeTrailingCall",
        "bad",
    );
    assert_code_fixture(
        &OperatorBeforeTrailingCall,
        "OperatorBeforeTrailingCall",
        "good",
    );
}

#[test]
fn given_forbidden_code_when_checked_then_reports_exact_node_lines() {
    assert_code_findings(
        &OperatorBeforeTrailingCall,
        "OperatorBeforeTrailingCall",
        "use std::ops::{Add, Mul};\nfn run() {\n    let result = (size * scale).round();\n    let result = (size + scale * other).max(1);\n    let result = size.mul(scale).add(other);\n}",
        &[3, 4, 5],
    );
}

#[test]
fn given_builder_chain_and_discarded_method_named_like_an_operator_when_checked_then_only_operators_report()
 {
    assert_code_findings(
        &OperatorBeforeTrailingCall,
        "OperatorBeforeTrailingCall",
        "use std::ops::{Add, Mul};\nstruct Order { items: Vec<Item> }\nimpl Order {\n    fn new() -> Self { Self { items: Vec::new() } }\n    fn add(mut self, item: Item) -> Self { self.items.push(item); self }\n}\nfn build(first: Item, second: Item) -> Order {\n    Order::new().add(first).add(second)\n}\nfn record(ledger: &mut Ledger, entry: Entry) {\n    ledger.add(entry);\n}\nfn total(size: u32, scale: u32, other: u32) -> u32 {\n    size.mul(scale).add(other)\n}\nfn price(unit: u32, quantity: u32) -> u32 {\n    unit.mul(quantity)\n}",
        &[14, 17],
    );
    assert_code_findings(
        &OperatorBeforeTrailingCall,
        "OperatorBeforeTrailingCall",
        "fn total(size: u32, scale: u32, other: u32) -> u32 {\n    size.mul(scale).add(other)\n}",
        &[],
    );
}

#[test]
fn given_lone_mul_call_when_std_ops_trait_is_imported_then_reports_the_call() {
    assert_code_findings(
        &OperatorBeforeTrailingCall,
        "OperatorBeforeTrailingCall",
        "use std::ops::Mul;\nfn run() {\n    let total = price.mul(quantity);\n}",
        &[3],
    );
    assert_code_findings(
        &OperatorBeforeTrailingCall,
        "OperatorBeforeTrailingCall",
        "fn run() {\n    let total = price.mul(quantity);\n}",
        &[],
    );
}

#[test]
fn given_prescribed_code_when_checked_then_passes() {
    assert_code_findings(
        &OperatorBeforeTrailingCall,
        "OperatorBeforeTrailingCall",
        "fn run() {\n    let result = size.mul(scale).round();\n    let result = size * (1 + scale);\n    let result = total.round();\n}",
        &[],
    );
}
