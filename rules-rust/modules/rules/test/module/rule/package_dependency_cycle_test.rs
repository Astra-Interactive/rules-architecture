use crate::check::api::Rule;
use crate::module::rule::PackageDependencyCycle;
use crate::test::check::usecase::read_case_context;
use crate::test::module::util::assert_resolution_cases;
use crate::test::source::syn::build_index;

#[test]
fn given_persistent_resolution_cases_when_checked_then_the_production_cycle_is_the_only_rule_finding()
 {
    assert_resolution_cases(
        &PackageDependencyCycle,
        "PackageDependencyCycle",
        "modules/sample/src/lib.rs:3\tfirst::model <-> second::model\n",
    );
}

#[test]
fn given_cycle_when_checked_then_one_component_names_both_physical_edges() {
    let mut context = read_case_context("ImportOrdering", "pass-syntax");
    context.indexes = vec![build_index(
        "mod first { pub mod model { pub struct First; use crate::second::model::Second; } } mod second { pub mod model { pub struct Second; use crate::first::model::First; } }",
    )];
    let findings = PackageDependencyCycle.check(&context);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].finding.message.contains("first::model"));
    assert!(findings[0].finding.message.contains("second::model"));
    assert!(findings[0].finding.message.contains("lib.rs:1"));
}

#[test]
fn given_mutually_exclusive_cfg_edges_when_checked_then_no_production_cycle_is_reported() {
    let mut context = read_case_context("ImportOrdering", "pass-syntax");
    for cfg in ["target_os = \"linux\"", "feature = \"fake\"", "test"] {
        context.indexes = vec![build_index(&format!(
            "mod first {{ pub mod model {{ pub struct First; #[cfg({cfg})] use crate::second::model::Second; }} }} mod second {{ pub mod model {{ pub struct Second; #[cfg(not({cfg}))] use crate::first::model::First; }} }}"
        ))];
        assert!(PackageDependencyCycle.check(&context).is_empty(), "{cfg}");
    }
}

#[test]
fn given_cycle_between_kind_packages_of_the_test_tree_when_checked_then_the_cycle_is_reported() {
    let mut context = read_case_context("ImportOrdering", "pass-syntax");
    context.indexes = vec![build_index(
        "#[cfg(test)]\nmod test {\n    pub mod first {\n        pub mod fake {\n            pub struct First;\n            use crate::test::second::fake::Second;\n        }\n    }\n    pub mod second {\n        pub mod fake {\n            pub struct Second;\n            use crate::test::first::fake::First;\n        }\n    }\n}",
    )];
    let findings = PackageDependencyCycle.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!("PackageDependencyCycle", findings[0].finding.rule_id.0);
    assert!(findings[0].finding.subject.contains("test::first::fake"));
    assert!(findings[0].finding.subject.contains("test::second::fake"));
}

#[test]
fn given_unix_and_windows_edges_when_checked_then_only_the_compatible_cycle_is_reported() {
    let mut context = read_case_context("ImportOrdering", "pass-syntax");
    context.indexes = vec![build_index(
        "mod clock { pub mod model { pub struct Tick; #[cfg(unix)] use crate::timer::model::Span; } } mod timer { pub mod model { pub struct Span; #[cfg(windows)] use crate::clock::model::Tick; } } mod third { pub mod model { pub struct Third; #[cfg(feature = \"enabled\")] use crate::fourth::model::Fourth; } } mod fourth { pub mod model { pub struct Fourth; #[cfg(feature = \"enabled\")] use crate::third::model::Third; } }",
    )];
    let findings = PackageDependencyCycle.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!("PackageDependencyCycle", findings[0].finding.rule_id.0);
    assert!(findings[0].finding.subject.contains("third::model"));
    assert!(findings[0].finding.subject.contains("fourth::model"));
    assert!(!findings[0].finding.subject.contains("clock"));
    assert!(!findings[0].finding.subject.contains("timer"));
}

#[test]
fn given_overlapping_cycles_when_checked_then_the_strong_component_is_reported_once() {
    let mut context = read_case_context("ImportOrdering", "pass-syntax");
    context.indexes = vec![build_index(
        "mod first { pub mod model { pub struct First; use crate::second::model::Second; use crate::third::model::Third; } } mod second { pub mod model { pub struct Second; use crate::first::model::First; } } mod third { pub mod model { pub struct Third; use crate::first::model::First; } }",
    )];
    let findings = PackageDependencyCycle.check(&context);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].finding.subject.contains("first::model"));
    assert!(findings[0].finding.subject.contains("second::model"));
    assert!(findings[0].finding.subject.contains("third::model"));
}

#[test]
fn given_conflicting_cycles_that_share_a_kind_when_checked_then_components_are_never_merged_across_cfgs()
 {
    let mut context = read_case_context("ImportOrdering", "pass-syntax");
    context.indexes = vec![build_index(
        "mod first { pub mod model { pub struct First; #[cfg(feature = \"left\")] use crate::second::model::Second; #[cfg(not(feature = \"left\"))] use crate::third::model::Third; } } mod second { pub mod model { pub struct Second; use crate::first::model::First; } } mod third { pub mod model { pub struct Third; use crate::first::model::First; } }",
    )];
    let findings = PackageDependencyCycle.check(&context);
    assert_eq!(findings.len(), 2);
    assert!(findings.iter().all(
        |finding| !(finding.finding.subject.contains("second::model")
            && finding.finding.subject.contains("third::model"))
    ));
}

#[test]
fn given_dense_return_paths_when_checked_then_only_cfg_compatible_cycles_are_reported() {
    let mut source = String::new();
    for node in 0..28 {
        source.push_str(&format!(
            "mod node_{node} {{ pub mod model {{ pub struct Node{node};"
        ));
        for target in node + 1..28 {
            if node == 0 {
                source.push_str("#[cfg(feature = \"enabled\")]");
            }
            source.push_str(&format!("use crate::node_{target}::model::Node{target};"));
        }
        if node == 27 {
            source.push_str("#[cfg(not(feature = \"enabled\"))] use crate::node_0::model::Node0;");
        }
        source.push_str("} }");
    }
    let mut context = read_case_context("ImportOrdering", "pass-syntax");
    context.indexes = vec![build_index(&source)];
    assert!(PackageDependencyCycle.check(&context).is_empty());
    let compatible = source.replace(
        "#[cfg(not(feature = \"enabled\"))]",
        "#[cfg(feature = \"enabled\")]",
    );
    context.indexes = vec![build_index(&compatible)];
    assert_eq!(PackageDependencyCycle.check(&context).len(), 1);
}

#[test]
fn given_alternative_cfg_paths_to_one_kind_when_checked_then_the_compatible_cycle_is_retained() {
    let mut context = read_case_context("ImportOrdering", "pass-syntax");
    context.indexes = vec![build_index(
        "mod first { pub mod model { pub struct First; #[cfg(feature = \"left\")] use crate::middle::model::Middle; #[cfg(not(feature = \"left\"))] use crate::middle::model::Middle as Other; } } mod middle { pub mod model { pub struct Middle; #[cfg(not(feature = \"left\"))] use crate::first::model::First; } }",
    )];
    let findings = PackageDependencyCycle.check(&context);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].finding.subject.contains("first::model"));
    assert!(findings[0].finding.subject.contains("middle::model"));
}

#[test]
fn given_disjoint_short_cycles_with_returning_bridges_when_checked_then_one_strong_component_is_reported()
 {
    let mut context = read_case_context("ImportOrdering", "pass-syntax");
    context.indexes = vec![build_index(
        "mod first { pub mod model { pub struct First; use crate::second::model::Second; } } mod second { pub mod model { pub struct Second; use crate::first::model::First; use crate::third::model::Third; } } mod third { pub mod model { pub struct Third; use crate::fourth::model::Fourth; } } mod fourth { pub mod model { pub struct Fourth; use crate::third::model::Third; use crate::first::model::First; } }",
    )];
    let findings = PackageDependencyCycle.check(&context);
    assert_eq!(findings.len(), 1);
    for concept in ["first", "second", "third", "fourth"] {
        assert!(
            findings[0]
                .finding
                .subject
                .contains(&format!("{concept}::model"))
        );
    }
}

#[test]
fn given_multiple_target_capabilities_when_checked_then_the_compatible_cycle_is_reported() {
    let mut context = read_case_context("ImportOrdering", "pass-syntax");
    for cfg in [
        "all(target_feature=\"sse2\",target_feature=\"avx\")",
        "all(target_has_atomic=\"8\",target_has_atomic=\"64\")",
        "all(target_family=\"unix\",target_family=\"wasm\")",
    ] {
        context.indexes = vec![build_index(&format!(
            "mod first {{ pub mod model {{ pub struct First; #[cfg({cfg})] use crate::second::model::Second; }} }} mod second {{ pub mod model {{ pub struct Second; use crate::first::model::First; }} }}"
        ))];
        assert_eq!(1, PackageDependencyCycle.check(&context).len(), "{cfg}");
    }
}
