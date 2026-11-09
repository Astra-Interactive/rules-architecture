use crate::check::api::Rule;
use crate::module::rule::DiPlacement;
use crate::test::check::usecase::read_case_context;
use crate::test::module::util::read_shape_context;

#[test]
fn given_contracts_only_crate_when_checked_then_di_block_is_reported_but_implementation_crate_passes()
 {
    let context = read_shape_context(
        "mod api { trait Contract {} } mod model { struct Value; } mod di {}",
        &[],
    );
    let findings = DiPlacement.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.subject, "case_sample::di");
    let context = read_shape_context(
        "mod api { trait Contract {} } mod file { struct Value; } mod di {}",
        &[],
    );
    assert!(DiPlacement.check(&context).is_empty());
}

#[test]
fn given_declaration_shapes_when_checked_then_exact_wrong_sites_report_and_correct_shapes_pass() {
    let context = read_case_context("DiPlacement", "fail-declaration-shape");
    let mut actual: Vec<String> = DiPlacement
        .check(&context)
        .iter()
        .map(|finding| {
            format!(
                "{}:{} {}",
                finding.finding.location.path,
                finding.finding.location.line.expect("finding line"),
                finding.finding.subject
            )
        })
        .collect();
    actual.sort();
    assert_eq!(
        actual,
        ["modules/sample/src/util/invoice_module.rs:1 case_sample::util::InvoiceModule"]
    );
    let context = read_case_context("DiPlacement", "pass-declaration-shape");
    assert!(DiPlacement.check(&context).is_empty());
}
#[test]
fn given_entrypoint_or_unknown_mechanisms_when_checked_then_contract_only_inference_is_not_vacuous()
{
    for source in [
        "fn main() {} mod di { struct RootModule; }",
        "mod legacy { struct Host; } mod di {}",
        "mod di {}",
    ] {
        let context = read_shape_context(source, &[]);
        assert!(DiPlacement.check(&context).is_empty(), "{source}");
    }
}

#[test]
fn given_dependency_assembly_without_module_when_checked_then_state_and_fakes_are_not_assembly() {
    let context = read_shape_context(
        "trait Repo {} struct FileRepo; impl Repo for FileRepo {} impl FileRepo { fn new() -> Self { Self } } fn build_owner() { let repo = FileRepo::new(); }",
        &[],
    );
    assert_eq!(DiPlacement.check(&context).len(), 1);
    let context = read_shape_context(
        "struct State; impl State { fn new() -> Self { Self } fn update(&mut self) {} } fn build_owner() { let state = State::new(); }",
        &[],
    );
    assert!(DiPlacement.check(&context).is_empty());
}
#[test]
fn given_owned_resource_lifecycle_when_checked_then_a_module_is_required_and_test_fakes_are_exempt()
{
    for (source, count) in [
        (
            "trait Lifecycle { fn on_disable(&self); } struct DatabasePool; impl Lifecycle for DatabasePool { fn on_disable(&self) {} }",
            1,
        ),
        (
            "trait Lifecycle { fn on_disable(&self); } struct DatabasePool; impl Lifecycle for DatabasePool { fn on_disable(&self) {} } mod di { struct DatabaseModule; }",
            0,
        ),
        (
            "trait Lifecycle { fn on_disable(&self); } #[cfg(test)] mod test { struct FakePool; impl super::Lifecycle for FakePool { fn on_disable(&self) {} } }",
            0,
        ),
    ] {
        assert_eq!(
            DiPlacement.check(&read_shape_context(source, &[])).len(),
            count,
            "{source}"
        );
    }
}
