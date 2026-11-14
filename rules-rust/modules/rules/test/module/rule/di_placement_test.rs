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
fn given_module_named_fakes_in_test_and_fixture_trees_when_checked_then_only_the_src_tree_is_judged()
 {
    let context = read_shape_context(
        "#[cfg(test)]\nmod test { mod flow { pub(crate) struct FakeWiringModule; impl FakeWiringModule { pub(crate) fn new() -> Self { Self } } } }\n#[cfg(any(test, feature = \"fake\"))]\npub mod fixture { pub mod host { pub mod fake { pub struct FakeHostModule; impl FakeHostModule { pub fn new() -> Self { Self } } } } }",
        &[],
    );
    let findings = DiPlacement.check(&context);
    assert!(findings.is_empty(), "{findings:?}");
    let context = read_shape_context(
        "mod flow { pub struct WiringModule; impl WiringModule { pub fn new() -> Self { Self } } }",
        &[],
    );
    let findings = DiPlacement.check(&context);
    assert_eq!(1, findings.len(), "{findings:?}");
    assert_eq!(
        "case_sample::flow::WiringModule",
        findings[0].finding.subject
    );
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
fn given_module_named_structs_outside_di_when_checked_then_only_those_with_di_shape_report() {
    let context = read_shape_context(
        "mod model { pub(crate) struct CourseModule { pub(crate) id: u64, pub(crate) title: String } }\nmod util {\n    pub(crate) struct InvoiceModule;\n    pub(crate) struct LedgerModule;\n    pub(crate) struct BillingModule { ledger_module: LedgerModule }\n    pub(crate) struct ReportModule;\n    impl ReportModule { pub(crate) fn new() -> Self { Self } }\n}",
        &[],
    );
    let mut subjects: Vec<String> = DiPlacement
        .check(&context)
        .into_iter()
        .map(|finding| finding.finding.subject)
        .collect();
    subjects.sort();
    assert_eq!(
        subjects,
        [
            "case_sample::util::BillingModule",
            "case_sample::util::ReportModule"
        ]
    );
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
fn given_di_shaped_type_outside_the_src_tree_when_checked_then_it_does_not_stand_in_for_the_di_module_of_src()
 {
    let assembly = "trait Repo {} struct FileRepo; impl Repo for FileRepo {} impl FileRepo { fn new() -> Self { Self } } fn build_owner() { let repo = FileRepo::new(); }";
    for tree in [
        "#[cfg(test)] mod test { mod di { pub struct FakeModule; } }",
        "#[cfg(any(test, feature = \"fake\"))] mod fixture { pub mod di { pub struct FakeModule; } }",
    ] {
        let context = read_shape_context(&format!("{assembly} {tree}"), &[]);
        let findings = DiPlacement.check(&context);
        assert_eq!(findings.len(), 1, "{tree}: {findings:?}");
        assert_eq!(findings[0].finding.subject, "case-sample");
    }
    let mut context = read_shape_context(
        &format!("{assembly} mod di {{ pub struct FakeModule; }}"),
        &[],
    );
    assert!(DiPlacement.check(&context).is_empty());
    for module in &mut context.indexes[0].modules {
        if module.module.path.last().is_some_and(|name| name == "di") {
            module.tree = crate::source::model::SourceTree::TestsIt;
        }
    }
    assert_eq!(DiPlacement.check(&context).len(), 1);
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
            "trait Lifecycle { fn on_disable(&self); } struct DatabasePool; impl Lifecycle for DatabasePool { fn on_disable(&self) {} } mod di { struct DatabaseModule { pool: super::DatabasePool } }",
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

#[test]
fn given_second_lifecycle_resource_no_di_item_owns_when_checked_then_only_that_resource_is_reported()
 {
    let context = read_shape_context(
        "trait Lifecycle { fn on_disable(&self); }\nstruct DatabasePool; impl Lifecycle for DatabasePool { fn on_disable(&self) {} }\nstruct CachePool; impl Lifecycle for CachePool { fn on_disable(&self) {} }\nmod di { struct DatabaseModule { pool: super::DatabasePool } }",
        &[],
    );
    let findings = DiPlacement.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].finding.subject, "case_sample::CachePool");
}

#[test]
fn given_di_module_in_the_library_of_a_binary_entry_point_when_main_constructs_it_then_the_crate_has_its_di_module()
 {
    let context = read_case_context("CrateTargets", "pass-binary-entry-point");
    assert!(DiPlacement.check(&context).is_empty());
}

#[test]
fn given_binary_main_assembling_an_implementation_when_no_target_has_a_di_module_then_the_crate_is_reported()
 {
    let context = crate::test::module::util::read_binary_shape_context(
        "pub trait Repo {}\npub struct FileRepo;\nimpl Repo for FileRepo {}\nimpl FileRepo { pub fn new() -> Self { Self } }",
        "use shop_cli::FileRepo;\nfn main() { let repo = FileRepo::new(); }",
    );
    let findings = DiPlacement.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].finding.subject, "shop_cli");
    assert_eq!(findings[0].finding.location.path, "shop_cli/src/main.rs");
}
