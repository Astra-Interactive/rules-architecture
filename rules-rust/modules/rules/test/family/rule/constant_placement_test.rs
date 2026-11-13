use crate::check::api::Rule;
use crate::family::rule::ConstantPlacement;
use crate::test::check::usecase::read_case_context;
use crate::test::module::util::{assert_placement_findings, read_placement_context};

#[test]
fn given_constant_holder_types_and_modules_when_checked_then_reports_them_beside_a_type() {
    crate::test::syntax::util::assert_syntax_findings(
        &ConstantPlacement,
        "ConstantOutsideImpl",
        "struct Subject; struct Defaults; impl Defaults { const VALUE: usize = 1; } enum Keys {} impl Keys { const KEY: usize = 1; } mod limits { const LIMIT: usize = 1; }",
        3,
    );
}

#[test]
fn given_trait_constants_statics_or_behaviour_when_checked_then_keeps_valid_items() {
    crate::test::syntax::util::assert_syntax_findings(
        &ConstantPlacement,
        "ConstantOutsideImpl",
        "const VALUE: usize = 1; static COUNT: usize = 1; trait Subject {} struct Other; impl Other { fn read(&self) {} } mod mixed { const VALUE: usize = 1; fn run() {} } mod empty {} mod external;",
        0,
    );
}

#[test]
fn given_constant_holder_in_its_own_file_when_checked_then_reports_the_holder_of_another_types_constants()
 {
    let context = read_placement_context(&[
        (
            "src/lib.rs",
            "pub(crate) mod invoice { pub(crate) mod file { pub(crate) use file_invoice_repository::FileInvoiceRepository; pub(crate) use file_invoice_repository_defaults::FileInvoiceRepositoryDefaults; pub(crate) use retry_policy::RetryPolicy; mod file_invoice_repository; mod file_invoice_repository_defaults; mod retry_policy; } }",
        ),
        (
            "src/invoice/file/file_invoice_repository.rs",
            "use crate::invoice::file::FileInvoiceRepositoryDefaults;\npub(crate) struct FileInvoiceRepository { flush_seconds: u64 }\nimpl FileInvoiceRepository { pub(crate) fn flush_seconds(&self) -> u64 { self.flush_seconds.max(FileInvoiceRepositoryDefaults::FLUSH_SECONDS) } }",
        ),
        (
            "src/invoice/file/file_invoice_repository_defaults.rs",
            "pub(crate) struct FileInvoiceRepositoryDefaults;\n\nimpl FileInvoiceRepositoryDefaults {\n    pub(crate) const FLUSH_SECONDS: u64 = 60;\n}\n",
        ),
        (
            "src/invoice/file/retry_policy.rs",
            "pub(crate) struct RetryPolicy { attempts: u32 }\n\nimpl RetryPolicy {\n    pub(crate) fn attempts(&self) -> u32 { self.attempts }\n    pub(crate) const MAX_ATTEMPTS: u32 = 3;\n}\n",
        ),
    ]);
    assert_placement_findings(
        &ConstantPlacement,
        "ConstantOutsideImpl",
        &context,
        &[(
            "src/invoice/file/file_invoice_repository_defaults.rs",
            1,
            "Node",
        )],
    );
}

#[test]
fn given_constant_only_file_beside_its_owner_type_when_checked_then_reports_the_file() {
    let context = read_placement_context(&[
        (
            "src/lib.rs",
            "pub(crate) mod feature_flag { pub(crate) mod di { pub(crate) use feature_flag_module::FeatureFlagModule; mod feature_flag_module; mod feature_flag_module_defaults; mod limits; mod parse_flag; } }",
        ),
        (
            "src/feature_flag/di/feature_flag_module.rs",
            "pub(crate) struct FeatureFlagModule { interval: u64 }\nimpl FeatureFlagModule { pub(crate) fn new() -> Self { Self { interval: super::feature_flag_module_defaults::RELOAD_SECONDS } } }",
        ),
        (
            "src/feature_flag/di/feature_flag_module_defaults.rs",
            "pub(crate) const RELOAD_SECONDS: u64 = 60;\n",
        ),
        (
            "src/feature_flag/di/limits.rs",
            "pub(crate) const MAX_FLAGS: usize = 64;\n",
        ),
        (
            "src/feature_flag/di/parse_flag.rs",
            "pub(crate) fn parse_flag(raw: &str) -> bool { raw == \"on\" && super::limits::MAX_FLAGS > 0 }",
        ),
    ]);
    assert_placement_findings(
        &ConstantPlacement,
        "ConstantOutsideImpl",
        &context,
        &[(
            "src/feature_flag/di/feature_flag_module_defaults.rs",
            1,
            "Node",
        )],
    );
}

#[test]
fn given_constant_only_file_whose_name_starts_with_a_type_file_name_when_checked_then_reports_it_only_on_a_word_boundary()
 {
    let context = read_placement_context(&[
        (
            "src/lib.rs",
            "pub(crate) mod endpoint { pub(crate) mod model { pub(crate) use port::Port; mod port; mod portal_paths; } }",
        ),
        ("src/endpoint/model/port.rs", "pub(crate) struct Port;\n"),
        (
            "src/endpoint/model/portal_paths.rs",
            "pub(crate) const ROOT: &str = \"/portal\";\n",
        ),
    ]);
    assert_placement_findings(&ConstantPlacement, "ConstantOutsideImpl", &context, &[]);
    let context = read_placement_context(&[
        (
            "src/lib.rs",
            "pub(crate) mod endpoint { pub(crate) mod model { pub(crate) use port::Port; mod port; mod port_limits; } }",
        ),
        ("src/endpoint/model/port.rs", "pub(crate) struct Port;\n"),
        (
            "src/endpoint/model/port_limits.rs",
            "pub(crate) const MAX_PORT: u16 = 65535;\n",
        ),
    ]);
    assert_placement_findings(
        &ConstantPlacement,
        "ConstantOutsideImpl",
        &context,
        &[("src/endpoint/model/port_limits.rs", 1, "Node")],
    );
}

#[test]
fn given_forbidden_syntax_when_checked_then_reports_the_rule() {
    let context = read_case_context("ConstantOutsideImpl", "fail-syntax");
    assert!(!ConstantPlacement.check(&context).is_empty());
}

#[test]
fn given_prescribed_syntax_when_checked_then_reports_nothing() {
    let context = read_case_context("ConstantOutsideImpl", "pass-syntax");
    assert!(ConstantPlacement.check(&context).is_empty());
}
