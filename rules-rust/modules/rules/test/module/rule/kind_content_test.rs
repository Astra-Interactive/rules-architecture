use crate::check::api::Rule;
use crate::module::rule::KindContent;
use crate::test::module::util::read_shape_context;

fn read_finding_lines(source: &str) -> Vec<Option<u32>> {
    let context = read_shape_context(source, &[]);
    let mut lines: Vec<Option<u32>> = KindContent
        .check(&context)
        .iter()
        .map(|finding| finding.finding.location.line)
        .collect();
    lines.sort_unstable();
    lines
}

#[test]
fn given_items_outside_the_rows_when_checked_then_reports_only_those_items() {
    let context = read_shape_context(
        "mod api { trait Reader {} struct Row; }\nmod model { struct Invoice; const LIMIT: u32 = 1; fn total() -> u32 { 0 } trait InvoiceFactory {} }\nmod usecase { struct ReadUseCase; enum Step { One } trait ReadContract {} }\nmod util { fn total() -> u32 { 0 } }",
        &[],
    );
    let mut subjects: Vec<String> = KindContent
        .check(&context)
        .iter()
        .map(|finding| finding.finding.subject.clone())
        .collect();
    subjects.sort();
    assert_eq!(
        subjects,
        [
            "case_sample::api::Row",
            "case_sample::model::total",
            "case_sample::usecase::Step"
        ]
    );
}

#[test]
fn given_api_with_struct_enum_union_function_const_and_static_when_checked_then_reports_each() {
    let lines = read_finding_lines(
        "mod api {\ntrait Reader {}\nstruct Row;\nenum Mode { One }\nunion Raw { value: u32 }\nfn load() {}\nconst LIMIT: u32 = 1;\nstatic NAME: u32 = 1;\n}",
    );
    assert_eq!(
        vec![Some(3), Some(4), Some(5), Some(6), Some(7), Some(8)],
        lines
    );
}

#[test]
fn given_model_with_free_function_and_static_when_checked_then_reports_both() {
    let lines = read_finding_lines(
        "mod model {\nstruct Invoice;\nconst LIMIT: u32 = 1;\ntype Outcome = u32;\ntrait InvoiceFactory {}\nfn total() -> u32 { 0 }\nstatic COUNT: u32 = 1;\n}",
    );
    assert_eq!(vec![Some(6), Some(7)], lines);
}

#[test]
fn given_usecase_with_enum_union_function_const_and_static_when_checked_then_reports_each() {
    let lines = read_finding_lines(
        "mod usecase {\nstruct ReadUseCase;\ntrait ReadContract {}\nenum Step { One }\nunion Raw { value: u32 }\nfn run() {}\nconst LIMIT: u32 = 1;\nstatic NAME: u32 = 1;\n}",
    );
    assert_eq!(vec![Some(4), Some(5), Some(6), Some(7), Some(8)], lines);
}

#[test]
fn given_modules_uses_macros_impls_and_other_kinds_when_checked_then_reports_nothing() {
    let lines = read_finding_lines(
        "mod api { use std::fmt::Debug; mod nested {} macro_rules! helper { () => {}; } trait Reader: Debug {} }\nmod model { struct Invoice; impl Invoice {} }\nmod usecase { struct ReadUseCase; impl ReadUseCase {} }\nmod util { fn total() -> u32 { 0 } }\nmod di { struct Container; }\nmod file { struct Row; enum Kind { One } }",
    );
    assert!(lines.is_empty(), "{lines:?}");
}
