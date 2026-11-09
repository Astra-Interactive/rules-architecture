#[test]
fn given_clock_constructions_when_checked_then_di_and_test_tree_are_exempt() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::SystemClockOutsideDi,
        "mod system { pub struct SystemClock; }\n fn build() { let clock = system::SystemClock; }\n mod di { fn build() { let clock = crate::system::SystemClock; } }\n #[cfg(test)] mod tests { fn build() { let clock = crate::system::SystemClock; } }",
        &[2],
    );
}

#[test]
fn given_clock_signatures_when_checked_then_exact_standard_time_and_shared_receiver_are_required() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::SystemClockOutsideDi,
        "mod system { struct SystemClock; } trait Clock { fn now(&self) -> std::time::SystemTime; }",
        &[],
    );
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::SystemClockOutsideDi,
        "struct SystemClock;\ntrait Clock { fn now(&mut self) -> std::time::SystemTime; }",
        &[1, 2],
    );
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::SystemClockOutsideDi,
        "trait Clock { fn now(&self) -> Result<std::time::SystemTime, ()>; }",
        &[1],
    );
}

#[test]
fn given_di_alias_shape_when_checked_then_preserves_defined_exceptions() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::SystemClockOutsideDi,
        "trait Clock { fn now(&self) -> &std::time::SystemTime; }",
        &[1],
    );
}

#[test]
fn given_persistent_di_policy_pairs_when_checked_then_exact_nodes_and_subjects_match() {
    for half in ["bad", "good"] {
        crate::test::module::util::assert_di_fixture(
            &crate::module::rule::SystemClockOutsideDi,
            "SystemClockOutsideDi",
            half,
        );
    }
}
