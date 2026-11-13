use crate::check::api::Rule;
use crate::config::model::Role;
use crate::module::rule::CoreShape;
use crate::test::module::util::read_shape_context;

#[test]
fn given_phase3_role_fixtures_when_checked_then_exact_wrong_sites_report_and_correct_roles_pass() {
    let context = crate::test::check::usecase::read_case_context("CoreShape", "fail-phase3-role");
    let mut actual: Vec<String> = CoreShape
        .check(&context)
        .iter()
        .map(|finding| {
            format!(
                "{}:{} {}",
                finding.finding.location.path,
                finding.finding.location.line.unwrap_or(0),
                finding.finding.subject
            )
        })
        .collect();
    actual.sort();
    assert_eq!(
        actual,
        [
            "modules/sample/src/api/invoice_repository.rs:1 case_sample::api::InvoiceRepository",
            "modules/sample/src/lib.rs:1 case_sample::usecase"
        ]
    );
    let context = crate::test::check::usecase::read_case_context("CoreShape", "pass-phase3-role");
    assert!(CoreShape.check(&context).is_empty());
}

#[test]
fn given_core_roles_when_checked_then_repository_and_usecase_kind_are_reported_in_each_role() {
    let mut context =
        read_shape_context("mod api { trait InvoiceRepository {} } mod usecase {}", &[]);
    assert!(CoreShape.check(&context).is_empty());
    for role in [Role::Core, Role::CoreApi, Role::CorePlatform, Role::CoreUi] {
        context.project_config.crates[0].role = Some(role);
        let mut subjects: Vec<String> = CoreShape
            .check(&context)
            .iter()
            .map(|finding| finding.finding.subject.clone())
            .collect();
        subjects.sort();
        assert_eq!(
            subjects,
            [
                "case_sample::api::InvoiceRepository",
                "case_sample::usecase"
            ]
        );
    }
}

#[test]
fn given_repository_types_in_core_roles_when_checked_then_struct_and_enum_are_reported() {
    let source = "mod file {\npub(crate) struct FileInvoiceRepository { folder: std::path::PathBuf }\npub(crate) enum StoreRepository { Memory }\npub(crate) struct ConfigReader { folder: std::path::PathBuf }\n}";
    let mut context = read_shape_context(source, &[]);
    assert!(CoreShape.check(&context).is_empty());
    for role in [Role::Core, Role::CoreApi, Role::CorePlatform, Role::CoreUi] {
        context.project_config.crates[0].role = Some(role);
        let mut sites: Vec<String> = CoreShape
            .check(&context)
            .iter()
            .map(|finding| {
                format!(
                    "{}:{} {}",
                    finding.finding.location.path,
                    finding.finding.location.line.unwrap_or(0),
                    finding.finding.subject
                )
            })
            .collect();
        sites.sort();
        assert_eq!(
            sites,
            [
                "modules/sample/src/lib.rs:2 case_sample::file::FileInvoiceRepository",
                "modules/sample/src/lib.rs:3 case_sample::file::StoreRepository"
            ]
        );
    }
}

#[test]
fn given_repository_trait_impl_in_core_roles_when_checked_then_reports_only_that_impl() {
    let source = "pub(crate) struct FileStore { folder: std::path::PathBuf }\nimpl vendor::invoice::InvoiceRepository for FileStore {}\nimpl std::fmt::Debug for FileStore { fn fmt(&self, sink: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { sink.write_str(\"FileStore\") } }";
    let mut context = read_shape_context(source, &[]);
    assert!(CoreShape.check(&context).is_empty());
    for role in [Role::Core, Role::CoreApi, Role::CorePlatform, Role::CoreUi] {
        context.project_config.crates[0].role = Some(role);
        let findings = CoreShape.check(&context);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].finding.rule_id.0, "CoreShape");
        assert_eq!(
            findings[0].finding.location.path,
            "modules/sample/src/lib.rs"
        );
        assert_eq!(findings[0].finding.location.line, Some(2));
    }
}

#[test]
fn given_repository_test_double_in_test_tree_when_checked_then_only_the_src_type_reports() {
    let source = "pub(crate) struct FileUserRepository { folder: std::path::PathBuf }\n#[cfg(test)]\nmod test {\n    pub(crate) struct FakeUserRepository;\n    impl vendor::user::UserRepository for FakeUserRepository {}\n}";
    let mut context = read_shape_context(source, &[]);
    assert!(CoreShape.check(&context).is_empty());
    for role in [Role::Core, Role::CoreApi, Role::CorePlatform, Role::CoreUi] {
        context.project_config.crates[0].role = Some(role);
        let findings = CoreShape.check(&context);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].finding.rule_id.0, "CoreShape");
        assert_eq!(
            findings[0].finding.location.path,
            "modules/sample/src/lib.rs"
        );
        assert_eq!(findings[0].finding.location.line, Some(1));
    }
}
