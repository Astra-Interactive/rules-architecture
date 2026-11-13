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

fn read_file_finding_subjects(source: &str, files: &[(&str, &str)]) -> Vec<String> {
    let context = read_shape_context(source, files);
    let mut subjects: Vec<String> = KindContent
        .check(&context)
        .iter()
        .map(|finding| finding.finding.subject.clone())
        .collect();
    subjects.sort();
    subjects
}

fn read_finding_subjects(source: &str) -> Vec<String> {
    read_file_finding_subjects(source, &[])
}

fn read_finding_messages(source: &str) -> Vec<String> {
    let context = read_shape_context(source, &[]);
    let mut messages: Vec<String> = KindContent
        .check(&context)
        .iter()
        .map(|finding| format!("{}: {}", finding.finding.subject, finding.finding.message))
        .collect();
    messages.sort();
    messages
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

#[test]
fn given_event_with_helper_struct_holding_a_method_when_checked_then_reports_the_helper() {
    let subjects = read_finding_subjects(
        "mod event {\nstruct ViewCache;\nimpl ViewCache { fn clear(&self) {} }\n}",
    );
    assert_eq!(subjects, ["case_sample::event::ViewCache"]);
}

#[test]
fn given_event_with_struct_implementing_api_trait_when_checked_then_reports_only_the_struct() {
    let subjects = read_finding_subjects(
        "mod api { trait ScopedJob { fn start(&self); } }\nmod event {\nuse crate::api::ScopedJob;\nstruct RefreshJob;\nimpl ScopedJob for RefreshJob { fn start(&self) {} }\n}",
    );
    assert_eq!(subjects, ["case_sample::event::RefreshJob"]);
}

#[test]
fn given_event_with_struct_implementing_only_drop_when_checked_then_reports_it() {
    let subjects = read_finding_subjects(
        "mod event {\nstruct ShutdownGuard;\nimpl Drop for ShutdownGuard { fn drop(&mut self) {} }\n}",
    );
    assert_eq!(subjects, ["case_sample::event::ShutdownGuard"]);
}

#[test]
fn given_event_with_helper_enum_and_union_when_checked_then_reports_each() {
    let source = "mod event {\nenum Mode { One }\nunion Raw { value: u32 }\n}";
    let subjects = read_finding_subjects(source);
    assert_eq!(
        subjects,
        ["case_sample::event::Mode", "case_sample::event::Raw"]
    );
}

#[test]
fn given_concept_event_with_helper_struct_when_checked_then_reports_it() {
    let subjects = read_finding_subjects(
        "mod tablist { mod event {\nstruct ViewCache;\nimpl ViewCache { fn clear(&self) {} }\n} }",
    );
    assert_eq!(subjects, ["case_sample::tablist::event::ViewCache"]);
}

#[test]
fn given_event_with_function_static_and_const_when_checked_then_reports_function_and_static() {
    let lines = read_finding_lines(
        "mod event {\nfn run() {}\nstatic NAME: u32 = 1;\nconst LIMIT: u32 = 1;\n}",
    );
    assert_eq!(vec![Some(2), Some(3)], lines);
}

#[test]
fn given_event_with_listener_and_helper_with_method_when_checked_then_reports_only_the_helper() {
    let subjects = read_finding_subjects(
        "mod event {\nstruct JoinListener;\nstruct ViewCache;\nimpl ViewCache { fn clear(&self) {} }\n}",
    );
    assert_eq!(subjects, ["case_sample::event::ViewCache"]);
}

#[test]
fn given_event_listener_implementing_a_trait_when_checked_then_reports_nothing() {
    let subjects = read_finding_subjects(
        "mod event {\nstruct JoinListener;\nimpl platform::Listener for JoinListener { fn on_join(&self) {} }\n}",
    );
    assert!(subjects.is_empty(), "{subjects:?}");
}

#[test]
fn given_event_listener_with_only_an_inherent_method_when_checked_then_reports_nothing() {
    let subjects = read_finding_subjects(
        "mod event {\nstruct PaidListener;\nimpl PaidListener { fn run(&self) {} }\n}",
    );
    assert!(subjects.is_empty(), "{subjects:?}");
}

#[test]
fn given_event_types_named_exactly_listener_and_event_when_checked_then_reports_nothing() {
    let subjects = read_finding_subjects(
        "mod event {\nstruct Listener;\nimpl Listener { fn run(&self) {} }\nstruct Event;\nimpl Event { fn fire(&self) {} }\n}",
    );
    assert!(subjects.is_empty(), "{subjects:?}");
}

#[test]
fn given_event_listener_with_plain_data_types_when_checked_then_reports_nothing() {
    let subjects = read_finding_subjects(
        "mod event {\nstruct JoinListener;\nstruct Settings { value: u32 }\nenum Mode { One }\n#[derive(Clone)]\nstruct Tag;\n}",
    );
    assert!(subjects.is_empty(), "{subjects:?}");
}

#[test]
fn given_event_listener_with_default_data_struct_when_checked_then_reports_the_struct() {
    let subjects = read_finding_subjects(
        "mod event {\nstruct JoinListener;\nstruct Settings;\nimpl Default for Settings { fn default() -> Self { Settings } }\n}",
    );
    assert_eq!(subjects, ["case_sample::event::Settings"]);
}

#[test]
fn given_event_types_named_with_event_suffix_when_checked_then_reports_nothing() {
    let source = "mod event {\nstruct RatedEvent;\nstruct PlayerJailedEvent;\n}";
    let subjects = read_finding_subjects(source);
    assert!(subjects.is_empty(), "{subjects:?}");
}

#[test]
fn given_event_helper_under_target_cfg_when_checked_then_reports_nothing() {
    let subjects = read_finding_subjects(
        "mod event {\n#[cfg(target_os = \"linux\")]\nstruct Helper;\n#[cfg(target_os = \"linux\")]\nimpl Helper { fn clear(&self) {} }\n}",
    );
    assert!(subjects.is_empty(), "{subjects:?}");
}

#[test]
fn given_helper_with_method_in_other_entry_kinds_when_checked_then_reports_nothing() {
    let subjects = read_finding_subjects(
        "mod job { struct ViewCache; impl ViewCache { fn clear(&self) {} } }\nmod service { struct ViewCache; impl ViewCache { fn clear(&self) {} } }\nmod command { struct ViewCache; impl ViewCache { fn clear(&self) {} } }\nmod route { struct ViewCache; impl ViewCache { fn clear(&self) {} } }\nmod warmup { struct ViewCache; impl ViewCache { fn clear(&self) {} } }\nmod migration { struct ViewCache; impl ViewCache { fn clear(&self) {} } }\nmod error { struct ViewCache; impl ViewCache { fn clear(&self) {} } }",
    );
    assert!(subjects.is_empty(), "{subjects:?}");
}

#[test]
fn given_event_with_trait_alias_use_mod_macro_and_listener_when_checked_then_reports_nothing() {
    let subjects = read_finding_subjects(
        "mod event {\nuse std::fmt::Debug;\nmod nested {}\nmacro_rules! helper { () => {}; }\ntrait Reader: Debug {}\ntype Outcome = u32;\nstruct JoinListener;\n}",
    );
    assert!(subjects.is_empty(), "{subjects:?}");
}

#[test]
fn given_event_data_struct_with_impl_of_excluded_feature_when_checked_then_reports_nothing() {
    let subjects = read_finding_subjects(
        "mod event {\nstruct JoinListener;\n#[cfg(feature = \"a\")]\nstruct Settings;\n#[cfg(not(feature = \"a\"))]\nimpl Settings { fn reset(&self) {} }\n}",
    );
    assert!(subjects.is_empty(), "{subjects:?}");
}

#[test]
fn given_event_struct_with_impl_of_same_feature_when_checked_then_reports_the_struct() {
    let subjects = read_finding_subjects(
        "mod event {\nstruct JoinListener;\n#[cfg(feature = \"a\")]\nstruct Settings;\n#[cfg(feature = \"a\")]\nimpl Settings { fn reset(&self) {} }\n}",
    );
    assert_eq!(subjects, ["case_sample::event::Settings"]);
}

#[test]
fn given_event_listener_and_data_struct_in_separate_files_when_checked_then_reports_the_struct() {
    let source = "mod event {\nmod join_listener;\nmod settings;\n}";
    let subjects = read_file_finding_subjects(
        source,
        &[
            ("event/join_listener.rs", "struct JoinListener;"),
            ("event/settings.rs", "struct Settings { value: u32 }"),
        ],
    );
    assert_eq!(subjects, ["case_sample::event::Settings"]);
}

#[test]
fn given_event_listener_in_nested_module_and_data_struct_when_checked_then_reports_the_struct() {
    let source = "mod event { mod nested { struct JoinListener; } struct Settings; }";
    let subjects = read_finding_subjects(source);
    assert_eq!(subjects, ["case_sample::event::Settings"]);
}

#[test]
fn given_event_data_struct_in_nested_module_and_listener_when_checked_then_reports_the_struct() {
    let source = "mod event { struct JoinListener; mod nested { struct Settings; } }";
    let subjects = read_finding_subjects(source);
    assert_eq!(subjects, ["case_sample::event::nested::Settings"]);
}

#[test]
fn given_listener_and_data_struct_in_two_concepts_when_checked_then_reports_the_struct() {
    let subjects = read_finding_subjects(
        "mod a { mod event { struct JoinListener; } }\nmod b { mod event { struct Settings; } }",
    );
    assert_eq!(subjects, ["case_sample::b::event::Settings"]);
}

#[test]
fn given_event_listener_with_const_only_and_empty_impls_when_checked_then_reports_nothing() {
    let subjects = read_finding_subjects(
        "mod event {\nstruct JoinListener;\nstruct Mode;\nimpl Mode { const A: u32 = 1; }\nstruct Flag;\nimpl Flag {}\n}",
    );
    assert!(subjects.is_empty(), "{subjects:?}");
}

#[test]
fn given_event_data_struct_with_impl_in_util_when_checked_then_reports_the_struct() {
    let subjects = read_finding_subjects(
        "mod event {\nstruct JoinListener;\nstruct Settings;\n}\nmod util {\nimpl crate::event::Settings { fn reset(&self) {} }\n}",
    );
    assert_eq!(subjects, ["case_sample::event::Settings"]);
}

#[test]
fn given_event_data_struct_with_cfg_test_impl_when_checked_then_reports_nothing() {
    let subjects = read_finding_subjects(
        "mod event {\nstruct JoinListener;\nstruct Settings;\n#[cfg(test)]\nimpl Settings { fn sample() {} }\n}",
    );
    assert!(subjects.is_empty(), "{subjects:?}");
}

#[test]
fn given_event_data_struct_with_impl_in_test_tree_when_checked_then_reports_nothing() {
    let subjects = read_finding_subjects(
        "mod event {\nstruct JoinListener;\nstruct Settings;\n}\n#[cfg(test)]\nmod test {\nmod event {\nimpl crate::event::Settings { fn sample() {} }\n}\n}",
    );
    assert!(subjects.is_empty(), "{subjects:?}");
}

#[test]
fn given_event_generic_tuple_struct_with_method_when_checked_then_reports_it() {
    let subjects = read_finding_subjects(
        "mod event {\nstruct Cache<T>(T);\nimpl<T> Cache<T> { fn get(&self) {} }\n}",
    );
    assert_eq!(subjects, ["case_sample::event::Cache"]);
}

#[test]
fn given_api_and_event_violations_when_checked_then_each_carries_its_own_message() {
    let messages = read_finding_messages(
        "mod api {\nstruct Row;\n}\nmod event {\nstruct ViewCache;\nimpl ViewCache { fn clear(&self) {} }\n}",
    );
    assert_eq!(
        messages,
        [
            "case_sample::api::Row: api holds contracts, model the level's types, usecase its operations; functions live in util (arch-package.md)",
            "case_sample::event::ViewCache: event holds listeners and platform event types: a job goes to job or service, state behind a contract in api, a flow of several steps to usecase, a platform effect to util (arch-package.md, Concepts and kinds 8)"
        ]
    );
}
