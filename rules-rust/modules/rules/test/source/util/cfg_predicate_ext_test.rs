use crate::source::model::CfgPredicate;

fn parse_predicate(text: &str) -> CfgPredicate {
    let item: syn::ItemStruct =
        syn::parse_str(&format!("#[cfg({text})] struct Value;")).expect("cfg syntax");
    CfgPredicate::from_attributes(&item.attrs)
}

#[test]
fn given_scoped_conditions_when_combined_then_local_flags_are_independent_and_targets_are_shared() {
    let feature = parse_predicate("feature = \"extra\"");
    let disabled = parse_predicate("not(feature = \"extra\")");
    assert!(
        !feature
            .in_scope(0)
            .and(&disabled.in_scope(0))
            .is_satisfiable()
    );
    assert!(
        feature
            .in_scope(0)
            .and(&disabled.in_scope(1))
            .is_satisfiable()
    );
    let test = parse_predicate("test");
    let production = parse_predicate("not(test)");
    assert!(
        !test
            .in_scope(0)
            .and(&production.in_scope(0))
            .is_satisfiable()
    );
    assert!(
        test.in_scope(0)
            .and(&production.in_scope(1))
            .is_satisfiable()
    );
    let linux = parse_predicate("target_os = \"linux\"");
    let windows = parse_predicate("target_os = \"windows\"");
    assert!(!linux.in_scope(0).and(&windows.in_scope(1)).is_satisfiable());
}

#[test]
fn given_cfg_conjunctions_when_checked_then_target_feature_and_test_conflicts_are_excluded() {
    for predicate in [
        "all(target_os = \"linux\", target_os = \"windows\")",
        "all(feature = \"fake\", not(feature = \"fake\"))",
        "all(test, not(test))",
        "not(any(test, not(test)))",
    ] {
        assert!(!parse_predicate(predicate).is_satisfiable(), "{predicate}");
    }
    for predicate in [
        "all(target_os = \"linux\", target_arch = \"aarch64\")",
        "any(test, not(test))",
        "all(feature = \"a\", feature = \"b\")",
        "all(target_feature = \"sse2\", target_feature = \"avx\")",
        "all(target_has_atomic = \"8\", target_has_atomic = \"64\")",
        "all(target_family = \"unix\", target_family = \"wasm\")",
        "all()",
        "not(any())",
    ] {
        assert!(parse_predicate(predicate).is_satisfiable(), "{predicate}");
    }
}
