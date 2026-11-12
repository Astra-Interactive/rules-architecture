use crate::check::api::Rule;
use crate::default::rule::StructDefaultValue;
use crate::test::check::usecase::read_case_context;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_standard_default_aliases_when_checked_then_reports_only_the_standard_trait() {
    assert_code_findings(
        &StructDefaultValue,
        "StructDefaultValue",
        "use std::default::Default as Initial;\nstruct Config;\nimpl Initial for Config { fn default() -> Self { Self } }\nmod custom { trait Default {} struct Value; impl Default for Value {} }\nmod nested { use core::default::Default as Start; struct Value; impl Start for Value { fn default() -> Self { Self } } }",
        &[3, 5],
    );
}

#[test]
fn given_struct_update_on_an_exempt_type_when_checked_then_only_the_plain_struct_reports() {
    assert_code_findings(
        &StructDefaultValue,
        "StructDefaultValue",
        "#[derive(serde::Deserialize, Default)]\npub struct Config { window: u16, symbol: String }\npub struct Cache { size: u16 }\nimpl Cache { pub fn new() -> Self { Self { size: 1 } } }\nimpl Default for Cache { fn default() -> Self { Self::new() } }\npub struct Plain { value: u64, other: u64 }\nfn build() {\n    let config = Config { window: 1, ..Default::default() };\n    let cache = Cache { size: 2, ..Default::default() };\n    let plain = Plain { value: 2, ..Default::default() };\n}",
        &[10],
    );
}

#[test]
fn given_persistent_code_fixtures_when_checked_then_match_exact_anchors() {
    assert_code_fixture(&StructDefaultValue, "StructDefaultValue", "bad");
    assert_code_fixture(&StructDefaultValue, "StructDefaultValue", "good");
}

#[test]
fn given_resolved_own_struct_updates_when_checked_then_reports_only_default_calls() {
    assert_code_findings(
        &StructDefaultValue,
        "StructDefaultValue",
        "mod model { pub struct Input { value: u64 } pub use Input as Alias; }\nfn build() {\n    let input = model::Alias { value: 2, ..Default::default() };\n    let input = model::Input { value: 2, ..model::Input::default() };\n    let input = external::Input { value: 2, ..Default::default() };\n    let input = model::Input { value: 2, ..previous };\n}",
        &[3, 4],
    );
}

#[test]
fn given_same_named_types_in_different_files_when_checked_then_constructor_exemption_stays_local() {
    let mut context = read_case_context("StructDefaultValue", "pass-syntax");
    let mut constructed = context.crates[0].files[1].clone();
    constructed.path = "constructed.rs".to_owned();
    constructed.syntax = syn::parse_file(
        "#[derive(Default)] struct Config; impl Config { fn new() -> Config { Config } }",
    )
    .expect("valid source");
    let mut data = constructed.clone();
    data.path = "data.rs".to_owned();
    data.syntax = syn::parse_file("#[derive(Default)] struct Config;").expect("valid source");
    context.crates[0].files = vec![constructed, data];
    let findings = StructDefaultValue.check(&context);
    assert_eq!(1, findings.len());
    assert_eq!("data.rs", findings[0].finding.location.path);
}

#[test]
fn given_non_delegating_default_bodies_when_checked_then_requires_the_exact_constructor_call() {
    for source in [
        "struct Subject; impl Subject { fn new() -> Self { Self } } impl Default for Subject { const VALUE: usize = 1; fn other() {} fn default() -> Self { Self } }",
        "struct Subject; impl Subject { fn new() -> Self { Self } } impl Default for Subject { fn default() -> Self { (factory())() } }",
    ] {
        crate::test::syntax::util::assert_syntax_findings(
            &StructDefaultValue,
            "StructDefaultValue",
            source,
            1,
        );
    }
    crate::test::syntax::util::assert_syntax_findings(
        &StructDefaultValue,
        "StructDefaultValue",
        "impl Default for (usize, usize) { fn default() -> Self { (0, 0) } }",
        0,
    );
}

#[test]
fn given_deserialization_and_zero_argument_construction_when_checked_then_accepts_defaults() {
    for source in [
        "#[derive(serde::Deserialize, Default)] struct Subject; impl Default for Subject { fn default() -> Self { Self } }",
        "struct Subject; impl Subject { fn new() -> Subject { Subject } } impl Default for Subject { fn default() -> Self { Subject::new() } }",
        "enum Subject {} impl Subject { fn new() -> Self { unreachable!() } } impl Default for Subject { fn default() -> Self { Self::new() } }",
        "impl Default for External { fn default() -> Self { Self } }",
    ] {
        crate::test::syntax::util::assert_syntax_findings(
            &StructDefaultValue,
            "StructDefaultValue",
            source,
            0,
        );
    }
}

#[test]
fn given_defaults_without_a_valid_constructor_delegation_when_checked_then_reports_them() {
    for source in [
        "struct Subject; impl Subject { fn new() -> Self { Self } } impl Default for Subject { fn default() -> Self { Self::new(1) } }",
        "#[derive(Default)] struct Subject; impl Subject { const VALUE: usize = 1; fn new() {} }",
        "struct Subject; impl Subject { fn new() -> Self { Self } } impl Default for Subject { fn default() -> Self { let value = Self; } }",
        "#[derive(Default)] enum Subject { #[default] Empty }",
        "struct Subject; impl Default for Subject { fn default() -> Self { Self } }",
        "struct Subject; impl Subject { fn new(value: usize) -> Self { Self } } impl Default for Subject { fn default() -> Self { Self::new(1) } }",
        "struct Subject; impl Subject { fn new() -> Self { Self } } impl Default for Subject { fn default() -> Self { Self::new(); Self } }",
        "mod nested { #[derive(Default)] struct Subject; }",
    ] {
        crate::test::syntax::util::assert_syntax_findings(
            &StructDefaultValue,
            "StructDefaultValue",
            source,
            1,
        );
    }
}

#[test]
fn given_derived_default_with_only_a_cfg_disabled_constructor_when_checked_then_reports_it() {
    assert_code_findings(
        &StructDefaultValue,
        "StructDefaultValue",
        "#[derive(Default)] struct Alpha;\n#[cfg(any())] impl Alpha { fn new() -> Self { Self } }\n#[derive(Default)] struct Beta;\nimpl Beta { #[cfg(any())] fn new() -> Self { Self } }\n#[derive(Default)] struct Gamma;\nimpl Gamma { fn new() -> Self { Self } }",
        &[1, 3],
    );
}

#[test]
fn given_default_derived_through_cfg_attr_when_checked_then_reports_it() {
    assert_code_findings(
        &StructDefaultValue,
        "StructDefaultValue",
        "#[derive(Debug, Clone)] #[cfg_attr(feature = \"defaults\", derive(Default))] struct Invoice { revision: u32, paid: bool }\n#[derive(Debug, Clone)] struct Plain { revision: u32, paid: bool }",
        &[1],
    );
}

#[test]
fn given_default_update_inside_a_select_handler_when_checked_then_reports_it() {
    assert_code_findings(
        &StructDefaultValue,
        "StructDefaultValue",
        "struct TickerConfig { retries: u8, limit: u8 }\npub(crate) async fn watch(mut tick: tokio::sync::mpsc::Receiver<u32>) {\n    tokio::select! {\n        value = tick.recv() => {\n            let config = TickerConfig { retries: 3, ..Default::default() };\n            drop(config);\n        }\n        else => {\n            let config = TickerConfig { retries: 3, limit: 1 };\n            drop(config);\n        }\n    }\n}",
        &[5],
    );
}

#[test]
fn given_forbidden_syntax_when_checked_then_reports_the_rule() {
    let context = read_case_context("StructDefaultValue", "fail-syntax");
    assert!(!StructDefaultValue.check(&context).is_empty());
}

#[test]
fn given_prescribed_syntax_when_checked_then_reports_nothing() {
    let context = read_case_context("StructDefaultValue", "pass-syntax");
    assert!(StructDefaultValue.check(&context).is_empty());
}
