use crate::check::api::Rule;
use crate::naming::rule::FunctionVerb;
use crate::test::check::usecase::read_case_context;

#[test]
fn given_lookup_names_when_checked_then_free_functions_report_and_collection_methods_keep_review_scope()
 {
    crate::test::syntax::util::assert_syntax_findings(
        &FunctionVerb,
        "FunctionVerb",
        "fn get() -> usize { 1 } struct Store { entries: std::collections::HashMap<u64, u64> } impl Store { fn get(&self, key: &u64) -> Option<&u64> { self.entries.get(key) } } struct Collection { values: Vec<u64> } impl Collection { fn get(&self, key: u64) -> Option<&u64> { self.values.iter().find(|value| **value == key) } fn get_user(&self) {} }",
        2,
    );
}

#[test]
fn given_trait_implementations_tests_and_lookups_when_checked_then_keeps_exempt_names() {
    crate::test::syntax::util::assert_syntax_findings(
        &FunctionVerb,
        "FunctionVerb",
        "fn get() {} fn get_mut() {} fn get_unchecked() {} fn get_unchecked_mut() {} fn forget_value() {} #[test] fn get_value() {} #[tokio::test] async fn map_value() {} impl Reader for Row { fn process_value() {} } trait Reader { fn handle_value(); } fn map_value() {} fn make_value() {} fn do_work() {} fn process_row() {}",
        9,
    );
}

#[test]
fn given_inherent_lookup_names_without_a_key_when_checked_then_reports_only_the_keyless_ones() {
    crate::test::declaration::util::assert_code_findings(
        &FunctionVerb,
        "FunctionVerb",
        "struct Account { balance: u64 }\nimpl Account {\n    fn get(&self) -> u64 { self.balance }\n    fn get_mut(&mut self) -> &mut u64 { &mut self.balance }\n    fn get_unchecked(&self) -> u64 { self.balance }\n    fn get_unchecked_mut(&mut self) -> &mut u64 { &mut self.balance }\n}\nstruct Store { entries: Vec<u64> }\nimpl Store {\n    fn get(&self, key: usize) -> Option<&u64> { self.entries.get(key) }\n    fn get_mut(&mut self, key: usize) -> Option<&mut u64> { self.entries.get_mut(key) }\n}\nimpl Reader for Account { fn get(&self) -> u64 { 0 } }\n#[test]\nfn get() {}",
        &[3, 4, 5, 6],
    );
}

#[test]
fn given_field_accessors_named_after_a_banned_verb_when_checked_then_only_non_accessors_report() {
    crate::test::declaration::util::assert_code_findings(
        &FunctionVerb,
        "FunctionVerb",
        "struct Schedule { handle: u32, process: Process, id: u32 }\nimpl Schedule {\n    fn handle(&self) -> u32 { self.handle }\n    fn process(&self) -> &Process { &self.process }\n    fn handle_id(&self) -> u32 { self.handle }\n    fn process_next(&self) -> u32 { self.id }\n    fn map(&self) -> u32 { self.handle + 1 }\n}",
        &[5, 6, 7],
    );
}

#[test]
fn given_forbidden_syntax_when_checked_then_reports_the_rule() {
    let context = read_case_context("FunctionVerb", "fail-syntax");
    assert!(!FunctionVerb.check(&context).is_empty());
}

#[test]
fn given_prescribed_syntax_when_checked_then_reports_nothing() {
    let context = read_case_context("FunctionVerb", "pass-syntax");
    assert!(FunctionVerb.check(&context).is_empty());
}
