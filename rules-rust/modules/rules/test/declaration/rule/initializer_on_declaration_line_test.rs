use crate::check::api::Rule;
use crate::check::di::CheckModule;
use crate::check::model::Report;
use crate::declaration::rule::InitializerOnDeclarationLine;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;
use crate::test::declaration::util::read_code_context;
use crate::test::suppression::util::build_workspace;
use std::fs;

fn read_subjects(source: &str) -> Vec<String> {
    let context = read_code_context(source);
    InitializerOnDeclarationLine
        .check(&context)
        .into_iter()
        .map(|finding| finding.finding.subject)
        .collect()
}

fn check_gate_source(source: &str) -> Report {
    let temporary = build_workspace();
    let source_directory = temporary.path().join("modules/sample/src");
    fs::write(source_directory.join("lib.rs"), "mod gate;").expect("write root source");
    fs::write(source_directory.join("gate.rs"), source).expect("write gate source");
    CheckModule::default()
        .check_workspace_use_case
        .invoke(temporary.path())
        .expect("valid workspace")
}

fn list_initializer_violations(report: &Report) -> Vec<&str> {
    report
        .violations
        .iter()
        .map(|finding| finding.rule_id.0.as_str())
        .filter(|rule_id| {
            *rule_id == "InitializerOnDeclarationLine" || rule_id.ends_with("Expectation")
        })
        .collect()
}

fn count_accepted_initializers(report: &Report) -> usize {
    report
        .accepted
        .iter()
        .filter(|exception| {
            exception.rule_id.0 == "InitializerOnDeclarationLine" && exception.reason == "x"
        })
        .count()
}

#[test]
fn given_persistent_code_fixtures_when_checked_then_match_exact_anchors() {
    assert_code_fixture(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        "bad",
    );
    assert_code_fixture(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        "good",
    );
}

#[test]
fn given_let_in_impl_method_broken_after_equals_when_checked_then_reports_let_line() {
    assert_code_findings(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        r#"impl Writer {
    fn write(&self, value: &Config) -> Result<(), ConfigurationError> {
        let content =
            serde_saphyr::to_string(value).map_err(|source| ConfigurationError::Serialize {
                path: self.path.clone(),
                source: Box::new(source),
            })?;
        Ok(())
    }
}"#,
        &[3],
    );
}

#[test]
fn given_lets_in_closure_and_async_block_broken_after_equals_when_checked_then_reports_both() {
    assert_code_findings(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        r#"fn run() {
    let task = || {
        let content =
            compute_content(first_argument);
        content
    };
    spawn(async move {
        let payload =
            build_payload(second_argument);
        payload
    });
}"#,
        &[3, 8],
    );
}

#[test]
fn given_typed_let_broken_after_equals_when_checked_then_reports_let_line() {
    assert_code_findings(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        r#"fn assemble() {
    let invoice_repository: Arc<dyn InvoiceRepository> =
        Arc::new(InMemoryInvoiceRepository::new());
}"#,
        &[2],
    );
}

#[test]
fn given_const_static_and_associated_const_broken_after_equals_when_checked_then_reports_each() {
    assert_code_findings(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        r#"const LIMIT: u32 =
    compute_limit();

static NAME: &str =
    "invoice";

struct Gate;

impl Gate {
    const WINDOW: Duration =
        Duration::from_secs(60);
}"#,
        &[1, 4, 10],
    );
}

#[test]
fn given_const_with_visibility_and_attribute_when_checked_then_anchors_at_keyword_line() {
    assert_code_findings(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        r#"#[cfg(unix)]
pub(crate) const LIMIT: u32 =
    compute_limit();"#,
        &[2],
    );
}

#[test]
fn given_let_else_broken_after_equals_when_checked_then_reports_let_line() {
    assert_code_findings(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        r#"fn find() {
    let Some(invoice) =
        find_invoice(id) else {
        return;
    };
}"#,
        &[2],
    );
}

#[test]
fn given_multiline_initializers_starting_on_equals_line_when_checked_then_passes() {
    assert_code_findings(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        r#"fn run() {
    let invoice = Invoice {
        id: first,
        total: second,
    };
    let names = receiver
        .first()
        .second();
    let handler = |value| {
        value
    };
    let label = match state {
        State::Open => "open",
        State::Closed => "closed",
    };
    let result = compute(
        first_argument,
        second_argument,
    );
    let Some(found) = lookup(id) else {
        return;
    };
}"#,
        &[],
    );
}

#[test]
fn given_const_and_static_initializers_starting_on_equals_line_when_checked_then_passes() {
    assert_code_findings(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        r#"const SETTINGS: Settings = Settings {
    first: 1,
    second: 2,
};

static NAMES: [&str; 2] = [
    "first",
    "second",
];

impl Gate {
    const WINDOW: Duration = Duration::from_secs(
        60,
    );
}"#,
        &[],
    );
}

#[test]
fn given_assignments_broken_after_operator_when_checked_then_passes() {
    assert_code_findings(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        r#"impl Holder {
    fn run(&mut self, value: u32) {
        self.cache =
            build_cache(first_argument);
        self.total +=
            value;
    }
}"#,
        &[],
    );
}

#[test]
fn given_let_without_initializer_when_checked_then_passes() {
    assert_code_findings(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        r#"fn run() {
    let value;
    let typed: u32;
    value = compute_value();
    typed = compute_typed();
}"#,
        &[],
    );
}

#[test]
fn given_generated_file_with_broken_let_when_checked_then_skips_it() {
    assert_code_findings(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        r#"// @generated by tool
fn run() {
    let value =
        compute_value(first_argument);
}"#,
        &[],
    );
    assert_code_findings(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        r#"// handwritten by a person
fn run() {
    let value =
        compute_value(first_argument);
}"#,
        &[3],
    );
}

#[test]
fn given_alias_broken_after_equals_when_checked_then_belongs_to_the_alias_rule() {
    assert_code_findings(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        r#"type KeepInventoryGate =
    FeatureGate<KeepInventoryModule, Reader>;

struct Feed;

impl Iterator for Feed {
    type Item =
        Pin<Box<Notification>>;
}"#,
        &[],
    );
}

#[test]
fn given_trait_const_default_broken_after_equals_when_checked_then_reports_const_line() {
    assert_code_findings(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        r#"trait Gate {
    const WINDOW: Duration =
        Duration::from_secs(60);
}"#,
        &[2],
    );
}

#[test]
fn given_trait_consts_without_default_or_with_default_on_equals_line_when_checked_then_passes() {
    assert_code_findings(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        r#"trait Gate {
    const SIZE: usize;
    const WINDOW: Duration = Duration::from_secs(60);
    const LIMITS: Limits = Limits {
        first: 1,
        second: 2,
    };
}"#,
        &[],
    );
}

#[test]
fn given_static_broken_after_equals_in_thread_local_body_when_checked_then_reports_static_line() {
    assert_code_findings(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        r#"thread_local! {
    static CACHE: RefCell<Cache> =
        RefCell::new(Cache::new());
}"#,
        &[2],
    );
}

#[test]
fn given_let_broken_after_equals_in_select_branch_when_checked_then_reports_let_line() {
    assert_code_findings(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        r#"async fn watch(mut tick: Receiver<u32>) {
    tokio::select! {
        value = tick.recv() => {
            let total =
                compute(value);
        }
        else => {
            let total = compute_default();
        }
    }
}"#,
        &[4],
    );
}

#[test]
fn given_let_broken_after_equals_in_vec_macro_argument_when_checked_then_reports_let_line() {
    assert_code_findings(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        r#"fn collect() {
    let items = vec![
        {
            let first =
                compute();
            first
        },
        second,
    ];
}"#,
        &[4],
    );
}

#[test]
fn given_let_broken_after_equals_in_vec_repeat_argument_when_checked_then_reports_let_line() {
    assert_code_findings(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        r#"fn run() {
    let items = vec![{
        let first =
            compute();
        first
    }; 3];
}"#,
        &[3],
    );
}

#[test]
fn given_let_mut_broken_after_equals_when_checked_then_subject_keeps_the_space() {
    let subjects = read_subjects("fn run() {\n    let mut total =\n        compute();\n}");

    assert_eq!(1, subjects.len(), "{subjects:?}");
    assert!(subjects[0].ends_with("let mut total"), "{subjects:?}");
}

#[test]
fn given_let_with_ref_binding_broken_after_equals_when_checked_then_subject_keeps_the_space() {
    let subjects = read_subjects(
        r#"fn run() {
    let Some(ref value) =
        lookup() else {
        return;
    };
}"#,
    );

    assert_eq!(1, subjects.len(), "{subjects:?}");
    assert!(subjects[0].contains("ref value"), "{subjects:?}");
}

#[test]
fn given_let_broken_after_equals_in_stream_macro_body_when_checked_then_reports_let_line() {
    assert_code_findings(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        r#"fn run() {
    let feed = stream! {
        let total =
            compute();
        yield total;
    };
}"#,
        &[3],
    );
}

#[test]
fn given_nested_brace_macros_when_checked_then_the_const_is_reported_once() {
    assert_code_findings(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        r#"outer! {
    inner! {
        const LIMIT: u8 =
            1;
    }
}"#,
        &[3],
    );
}

#[test]
fn given_expected_select_statement_around_broken_let_when_checked_then_the_exception_is_accepted() {
    let report = check_gate_source(
        r#"async fn watch(mut tick: Receiver<u32>) {
    #[cfg_attr(klibs_rules, expect(InitializerOnDeclarationLine, reason = "x"))]
    tokio::select! {
        value = tick.recv() => {
            let total =
                compute(value);
        }
    }
}"#,
    );

    assert_eq!(Vec::<&str>::new(), list_initializer_violations(&report));
    assert_eq!(1, count_accepted_initializers(&report));
}

#[test]
fn given_expected_thread_local_item_around_broken_static_when_checked_then_it_is_accepted() {
    let report = check_gate_source(
        r#"#[cfg_attr(klibs_rules, expect(InitializerOnDeclarationLine, reason = "x"))]
thread_local! {
    static CACHE: RefCell<Cache> =
        RefCell::new(Cache::new());
}"#,
    );

    assert_eq!(Vec::<&str>::new(), list_initializer_violations(&report));
    assert_eq!(1, count_accepted_initializers(&report));
}

#[test]
fn given_expected_let_in_plain_block_when_checked_then_the_exception_is_accepted() {
    let report = check_gate_source(
        r#"fn run() {
    {
        #[cfg_attr(klibs_rules, expect(InitializerOnDeclarationLine, reason = "x"))]
        let total =
            compute();
    }
}"#,
    );

    assert_eq!(Vec::<&str>::new(), list_initializer_violations(&report));
    assert_eq!(1, count_accepted_initializers(&report));
}

#[test]
fn given_other_rule_expectation_inside_select_handler_when_checked_then_it_is_not_unfulfilled() {
    let report = check_gate_source(
        r#"fn f() {
    tokio::select! {
        _ = a => {
            #[cfg_attr(klibs_rules, expect(LocalItem, reason = "r"))]
            struct Row;
        }
    }
}"#,
    );

    assert_eq!(Vec::<&str>::new(), list_initializer_violations(&report));
}

#[test]
fn given_thread_local_without_trailing_semicolon_in_braces_when_checked_then_reports_static() {
    assert_code_findings(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        r#"thread_local! {
    static CACHE: RefCell<u32> =
        RefCell::new(0)
}"#,
        &[2],
    );
}

#[test]
fn given_thread_local_without_trailing_semicolon_in_parens_when_checked_then_reports_static() {
    assert_code_findings(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        r#"thread_local!(
    static CACHE: RefCell<u32> =
        RefCell::new(0)
);"#,
        &[2],
    );
}

#[test]
fn given_macro_body_that_is_one_broken_let_when_checked_then_reports_let_line() {
    assert_code_findings(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        r#"fn f() {
    m! {
        let total =
            compute()
    }
}"#,
        &[3],
    );
}

#[test]
fn given_macro_body_let_with_operator_initializer_when_checked_then_reports_let_line() {
    for initializer in ["a || b", "a && b", "0..n"] {
        let source = format!(
            "fn f() {{\n    m! {{\n        let ok =\n            {initializer}\n    }}\n}}"
        );

        assert_code_findings(
            &InitializerOnDeclarationLine,
            "InitializerOnDeclarationLine",
            &source,
            &[3],
        );
    }
}

#[test]
fn given_macro_body_let_with_operator_initializer_and_tail_when_checked_then_reports_let_line() {
    for initializer in ["a || b", "a && b", "0..n"] {
        let source = format!(
            "fn f() {{\n    m! {{\n        let ok =\n            {initializer};\n        ok\n    }}\n}}"
        );

        assert_code_findings(
            &InitializerOnDeclarationLine,
            "InitializerOnDeclarationLine",
            &source,
            &[3],
        );
    }
}

#[test]
fn given_macro_body_with_statement_before_broken_let_when_checked_then_reports_let_line() {
    assert_code_findings(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        r#"fn f() {
    m! {
        prepare();
        let total =
            compute()
    }
}"#,
        &[4],
    );
}

#[test]
fn given_declarations_broken_after_equals_in_inline_module_when_checked_then_reports_each_line() {
    assert_code_findings(
        &InitializerOnDeclarationLine,
        "InitializerOnDeclarationLine",
        r#"mod gates {
    const LIMIT: u32 =
        compute_limit();

    static NAME: &str =
        "invoice";

    fn build() {
        let value =
            compute_value();
    }
}"#,
        &[2, 5, 9],
    );
}

#[test]
fn given_declarations_broken_after_equals_in_inline_module_when_checked_then_subjects_carry_path() {
    let subjects = read_subjects(
        r#"mod gates {
    const LIMIT: u32 =
        compute_limit();

    static NAME: &str =
        "invoice";

    fn build() {
        let value =
            compute_value();
    }
}"#,
    );

    assert_eq!(3, subjects.len(), "{subjects:?}");
    for expected in [
        "::gates const LIMIT",
        "::gates static NAME",
        "::gates let value",
    ] {
        assert!(
            subjects.iter().any(|subject| subject.ends_with(expected)),
            "{expected}: {subjects:?}"
        );
    }
}
