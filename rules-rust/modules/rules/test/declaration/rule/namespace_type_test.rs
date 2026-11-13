use crate::declaration::rule::NamespaceType;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_persistent_code_fixtures_when_checked_then_match_exact_anchors() {
    assert_code_fixture(&NamespaceType, "NamespaceType", "bad");
    assert_code_fixture(&NamespaceType, "NamespaceType", "good");
}

#[test]
fn given_alias_and_cfg_separated_trait_impl_when_checked_then_reports_uncovered_namespace_branch() {
    assert_code_findings(
        &NamespaceType,
        "NamespaceType",
        "struct Namespace;\nuse Namespace as Alias;\n#[cfg(feature = \"active\")] impl Alias { fn run() {} }\n#[cfg(not(feature = \"active\"))] impl Contract for Namespace {}",
        &[1],
    );
    assert_code_findings(
        &NamespaceType,
        "NamespaceType",
        "mod left { struct Subject; impl Subject { fn run() {} } impl Contract for Subject {} }\nmod right { struct Subject; impl Subject { fn run() {} } }",
        &[2],
    );
}

#[test]
fn given_forbidden_code_when_checked_then_reports_exact_node_lines() {
    assert_code_findings(
        &NamespaceType,
        "NamespaceType",
        "mod model {\n    struct Functions;\n    impl Functions { fn run() {} }\n    enum Empty {}\n    impl Empty { fn run() {} }\n}",
        &[2, 4],
    );
}

#[test]
fn given_prescribed_code_when_checked_then_passes() {
    assert_code_findings(
        &NamespaceType,
        "NamespaceType",
        "mod model {\n    struct Stateful { value: bool }\n    impl Stateful { fn run() {} }\n    struct Rule;\n    impl Rule { fn run() {} }\n    impl Contract for Rule {}\n    struct Constants;\n    impl Constants { const VALUE: bool = true; }\n}",
        &[],
    );
}
