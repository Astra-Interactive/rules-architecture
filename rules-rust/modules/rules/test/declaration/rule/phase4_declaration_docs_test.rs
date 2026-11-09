use crate::check::api::Rule;
use crate::check::model::{Anchor, RuleContext};
use crate::comment::rule::{DocCommentScope, IntraDocLink, RustdocFiller};
use crate::declaration::rule::{BareIdentifier, CallbackDeclaration};
use crate::expression::rule::TrivialClosurePath;
use crate::module::rule::{NestedDataStruct, NestedEnum};
use crate::test::declaration::util::read_code_context;
use crate::test::module::util::read_shape_context;
use crate::tuple::rule::AnonymousTuple;

fn assert_lines(rule: &dyn Rule, context: &RuleContext, expected: &[u32]) {
    assert!(
        context
            .indexes
            .iter()
            .all(|index| index.unanalysable.is_empty()),
        "unexpected source blind spot"
    );
    let findings = rule.check(context);
    let mut actual: Vec<_> = findings
        .iter()
        .map(|finding| {
            let Anchor::Node { file, span } = &finding.anchor else {
                panic!("Node expected")
            };
            assert_eq!(&finding.finding.location.path, file);
            assert_eq!(finding.finding.location.line, Some(span.start.line));
            span.start.line
        })
        .collect();
    actual.sort_unstable();
    assert_eq!(expected, actual, "{findings:?}");
}

fn persistent_context(name: &str, half: &str) -> (RuleContext, String) {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../cases/code")
        .join(name);
    let stem = format!("phase4-declaration-{half}");
    if ["NestedDataStruct", "NestedEnum"].contains(&name) {
        let directory = directory.join(stem);
        let source = std::fs::read_to_string(directory.join("src/lib.rs")).expect("root fixture");
        let mut paths = vec!["model/owner.rs".to_owned()];
        if half == "bad" {
            paths.push(format!(
                "model/{}.rs",
                if name == "NestedEnum" {
                    "status"
                } else {
                    "payload"
                }
            ));
        }
        let files: Vec<_> = paths
            .iter()
            .map(|path| {
                (
                    path.as_str(),
                    std::fs::read_to_string(directory.join("src").join(path))
                        .expect("file fixture"),
                )
            })
            .collect();
        let references: Vec<_> = files
            .iter()
            .map(|(path, source)| (*path, source.as_str()))
            .collect();
        return (
            read_shape_context(&source, &references),
            std::fs::read_to_string(directory.join("expected.tsv")).expect("expectations"),
        );
    }
    let source = std::fs::read_to_string(directory.join(format!("{stem}.rs"))).expect("fixture");
    let mut context = read_code_context(&source);
    if name == "DocCommentScope" {
        context.workspace.members[0].manifest = crate::workspace::util::parse_toml_file(
            "Cargo.toml",
            "[package]\nname=\"case-sample\"\nversion=\"0.1.0\"",
        )
        .expect("version");
    }
    (
        context,
        std::fs::read_to_string(directory.join(format!("{stem}.tsv"))).expect("expectations"),
    )
}

#[test]
fn given_real_renamed_workspace_consumers_when_nested_rules_checked_then_public_reexport_users_prevent_false_exclusivity()
 {
    use crate::source::api::SourceReader;
    use crate::workspace::api::WorkspaceReader;
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../cases/code/NestedDataStruct/workspace");
    let mut context = read_code_context("");
    context.workspace = crate::workspace::cargo::CargoWorkspaceReader::new(
        crate::workspace::util::resolve_cargo_path(),
    )
    .read_workspace(&directory)
    .expect("workspace");
    context.project_config =
        crate::config::util::parse_project_config(&context.workspace).expect("configuration");
    context.crates = crate::source::syn::SynSourceReader.read_sources(&context.workspace);
    context.indexes =
        crate::source::syn::CrateIndexBuilder::build_all(&context.crates, &context.workspace);
    assert!(
        context
            .indexes
            .iter()
            .all(|index| index.unanalysable.is_empty())
    );
    let findings = NestedDataStruct
        .check(&context)
        .into_iter()
        .chain(NestedEnum.check(&context));
    let mut actual: Vec<_> = findings
        .map(|finding| {
            format!(
                "{}\t{}:{}\tNode\t{}",
                finding.finding.rule_id.0,
                finding.finding.location.path,
                finding.finding.location.line.expect("line"),
                finding.finding.subject
            )
        })
        .collect();
    actual.sort();
    let expected = std::fs::read_to_string(directory.join("expected.tsv")).expect("expectations");
    assert_eq!(expected.trim_end(), actual.join("\n"));
}

#[test]
fn given_persistent_phase4_declaration_pairs_when_checked_then_ids_subjects_and_node_anchors_match()
{
    let rules: &[(&str, &dyn Rule)] = &[
        ("BareIdentifier", &BareIdentifier),
        ("CallbackDeclaration", &CallbackDeclaration),
        ("AnonymousTuple", &AnonymousTuple),
        ("RustdocFiller", &RustdocFiller),
        ("TrivialClosurePath", &TrivialClosurePath),
        ("DocCommentScope", &DocCommentScope),
        ("NestedDataStruct", &NestedDataStruct),
        ("NestedEnum", &NestedEnum),
        ("IntraDocLink", &IntraDocLink),
    ];
    for (name, rule) in rules {
        for half in ["bad", "good"] {
            let (context, expected) = persistent_context(name, half);
            assert!(
                context
                    .indexes
                    .iter()
                    .all(|index| index.unanalysable.is_empty())
            );
            let mut actual: Vec<_> = rule
                .check(&context)
                .iter()
                .map(|finding| {
                    let Anchor::Node { file, span } = &finding.anchor else {
                        panic!("node")
                    };
                    assert_eq!(finding.finding.location.path, *file);
                    assert_eq!(finding.finding.location.line, Some(span.start.line));
                    format!(
                        "{}\t{}:{}\tNode\t{}",
                        finding.finding.rule_id.0, file, span.start.line, finding.finding.subject
                    )
                })
                .collect();
            actual.sort();
            let mut expected: Vec<_> = expected
                .lines()
                .filter(|line| !line.is_empty())
                .map(str::to_owned)
                .collect();
            expected.sort();
            assert_eq!(expected, actual, "{name}/{half}");
        }
    }
}

#[test]
fn given_single_model_fields_when_checked_then_tuple_serde_and_nonmodel_types_are_distinguished() {
    assert_lines(
        &BareIdentifier,
        &read_code_context(
            "mod model {\nstruct Money { cents: i64 }\nstruct Valid(i64); #[derive(serde::Serialize)] struct Wire { cents: i64 } } mod file { struct Row { value: i64 } }",
        ),
        &[2, 3],
    );
}

#[test]
fn given_callback_fields_when_checked_then_real_aliases_and_generic_bounds_report_but_collections_and_local_fn_pass()
 {
    let source = "type Completion<T: ?Sized> = Box<T>;\nstruct Worker { handler: Completion<dyn Fn()> }\ntrait TimerListener {}\nstruct Timer { listener: std::sync::Arc<dyn TimerListener> }\nstruct Generic<F: Fn()> { handler: F }\nstruct Collection { handlers: Vec<fn()> }\nmod local { trait Fn {} struct Local { handler: Box<dyn Fn> } }\n#[cfg(test)] mod tests { struct Worker { handler: Box<dyn std::ops::Fn()> } }";
    assert_lines(
        &CallbackDeclaration,
        &read_code_context(source),
        &[2, 3, 4, 5],
    );
}

#[test]
fn given_tuple_bindings_when_checked_then_destructuring_and_named_channel_values_pass() {
    assert_lines(
        &AnonymousTuple,
        &read_code_context(
            "fn work() {\nlet tuple = (1, 2);\nlet (head, tail) = (1, 2);\nlet channels = tokio::sync::mpsc::channel::<u64>(8);\nlet named = channel();\n} struct Channel { sender: u64, receiver: u64 } fn channel() -> Channel { todo!() }",
        ),
        &[2, 4],
    );
    assert_lines(
        &AnonymousTuple,
        &read_code_context(
            "struct Parts { head: String, tail: String } struct Buffer(String); impl Buffer { fn split_at(&self, index: usize) -> Parts { todo!() } } fn work(buffer: Buffer) { let parts = buffer.split_at(1); }",
        ),
        &[],
    );
}

#[test]
fn given_parameter_docs_when_checked_then_only_actual_parameter_word_restatement_reports() {
    assert_lines(
        &RustdocFiller,
        &read_code_context(
            "/// `user_id` — the user id.\nfn load(user_id: u64) {}\n/// `timeout` — seconds before retrying.\nfn wait(timeout: u64) {}\n/// `other` — the other.\nfn ignored(value: u64) {}",
        ),
        &[1],
    );
}

#[test]
fn given_path_adapter_chains_when_checked_then_only_maximal_three_step_runs_report() {
    assert_lines(
        &TrivialClosurePath,
        &read_code_context(
            "fn work(values: Vec<u64>) {\nlet result = values.iter().map(first).map(second).map(third).map(fourth).collect();\nlet valid = values.iter().map(first).map(second).collect();\nlet broken = values.iter().map(first).take(2).map(second).map(third);\n} fn first(value: u64) -> u64 { value } fn second(value: u64) -> u64 { value } fn third(value: u64) -> u64 { value } fn fourth(value: u64) -> u64 { value }",
        ),
        &[2],
    );
}

#[test]
fn given_two_step_string_adaptation_when_checked_then_only_equivalent_function_paths_report() {
    assert_lines(
        &TrivialClosurePath,
        &read_code_context(
            "fn parse_order(value: &str) -> usize { value.len() }\nfn run(text: &str, custom: Custom) {\nlet values = text.lines().map(|line| parse_order(line.trim()));\nlet paths = text.lines().map(str::trim).map(parse_order);\nlet captured = text.lines().map(|line| custom.parse(line.trim()));\nlet three = text.lines().map(|line| parse_order(line.trim().trim()));\nlet different = custom.lines().map(|line| parse_order(line.trim()));\nlet field = text.lines().map(|line| parse_order(line.value.trim()));\nlet repeated = text.lines().map(|line| parse_order(combine(line.trim(), line)));\n{ let text = custom; let named = text.lines().map(|line| parse_order(line.trim())); }\nlet final_values = text.lines().map(|line| parse_order(line.trim()));\n}\nstruct Custom; fn combine<'text>(first: &'text str, second: &'text str) -> &'text str { first }",
        ),
        &[3, 11],
    );
}

#[test]
fn given_doc_scope_rules_when_checked_then_impl_docs_labels_and_published_panic_contracts_are_precise()
 {
    let mut context = read_code_context(
        "trait Contract { fn run(&self); } struct Handler; impl Contract for Handler {\n/// # Errors\n/// Never fails.\nfn run(&self) {} }\n/// Errors: not a heading.\nfn wrong() {}\npub fn panic_on_input(value: bool) { assert!(value); }\n/// # Panics\n/// Panics on missing input.\npub fn valid(value: bool) { assert!(value); }",
    );
    context.workspace.members[0].manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[package]\nname=\"case-sample\"\nversion=\"0.1.0\"\n",
    )
    .expect("published metadata");
    assert_lines(&DocCommentScope, &context, &[2, 5, 7]);
    context = read_code_context(
        "/// Wrong.\nfn wrong() {}\n/// # Errors\n/// Returns [`Failure`] for missing input.\nfn valid() {}\n/// Platform issue: [details](https://example.invalid/issue).\nfn reason() {}",
    );
    context.workspace.members[0].manifest =
        crate::workspace::util::parse_toml_file("Cargo.toml", "[package]\nname=\"case-sample\"\n")
            .expect("application metadata");
    assert_lines(&DocCommentScope, &context, &[1]);
}

#[test]
fn given_trim_variants_and_unsafe_callees_when_checked_then_recommends_only_safe_function_paths() {
    assert_lines(
        &TrivialClosurePath,
        &read_code_context(
            "fn parse_line(line: &str) -> usize { line.len() }\nunsafe fn parse_unsafe(line: &str) -> usize { line.len() }\n#[target_feature(enable = \"sse2\")] unsafe fn parse_feature(line: &str) -> usize { line.len() }\nfn parse_generic<T: Default>(line: &str) -> T { T::default() }\nfn run(text: &str) {\nlet start = text.lines().map(|line| parse_line(line.trim_start()));\nlet end = text.lines().map(|line| parse_line(line.trim_end()));\nlet unsafe_call = unsafe { text.lines().map(|line| parse_unsafe(line.trim())).count() };\nlet feature_call = unsafe { text.lines().map(|line| parse_feature(line.trim())).count() };\nlet generic = text.lines().map(|line| parse_generic::<usize>(line.trim()));\n}",
        ),
        &[6, 7],
    );
}

#[test]
fn given_safe_target_feature_callees_when_checked_then_retains_closures_that_enable_the_call() {
    assert_lines(
        &TrivialClosurePath,
        &read_code_context(
            "#[target_feature(enable = \"sse2\")] fn parse_feature(line: &str) -> usize { line.len() }\n#[cfg_attr(target_arch = \"x86_64\", target_feature(enable = \"sse2\"))] fn parse_conditional(line: &str) -> usize { line.len() }\nfn run(text: &str) {\nlet feature_call = unsafe { text.lines().map(|line| parse_feature(line.trim())).count() };\nlet conditional_call = unsafe { text.lines().map(|line| parse_conditional(line.trim())).count() };\n}",
        ),
        &[],
    );
}

#[test]
fn given_published_unwrap_and_custom_expect_when_checked_then_only_known_panicking_receivers_require_panics()
 {
    let mut context = read_code_context(
        "pub fn value(input: Option<u64>) -> u64 { input.unwrap() }\npub struct Counter; impl Counter { pub fn expect(&self) -> u64 { 1 } }\npub fn count(counter: &Counter) -> u64 { counter.expect() }",
    );
    context.workspace.members[0].manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[package]\nname=\"case-sample\"\nversion=\"0.1.0\"",
    )
    .expect("version");
    assert_lines(&DocCommentScope, &context, &[1]);
}

#[test]
fn given_doc_code_spans_when_checked_then_only_resolved_unlinked_candidates_report() {
    assert_lines(
        &IntraDocLink,
        &read_code_context(
            "struct Order; impl Order { fn load() {} }\n/// `Order` and `Order::load`, but `Order::missing` and `Unknown`.\n/// [`Order`] already links it.\n/// ```rust\n/// `Order` in a fenced example.\n/// ```\n/// Escaped \\`Order\\` stays text.\nfn work() {}",
        ),
        &[2, 2],
    );
}

#[test]
fn given_doc_self_members_and_private_scope_when_checked_then_real_member_identity_is_used() {
    assert_lines(
        &IntraDocLink,
        &read_code_context(
            "mod model { pub struct Order; }\nmod api { use crate::model::Order as Purchase;\n/// `Purchase` resolves this alias.\nfn helper() {} }\nstruct Handler; impl Handler {\n/// `Self::run` exists and `Self::missing` does not.\nfn run(&self) {} }",
        ),
        &[3, 6],
    );
}

#[test]
fn given_doc_alias_members_escaping_and_cfg_when_checked_then_markdown_and_semantic_scopes_are_distinct()
 {
    assert_lines(
        &IntraDocLink,
        &read_code_context(
            "struct Order; impl Order { fn load() {} } type Purchase = Order;\n/// ``Purchase::load`` exists.\n/// [the `Order` type](https://example.invalid) already links it.\n/// \\[ `Order` ] is an escaped bracket, not a link.\n/// ```Order``` is an inline span, not a fence info string.\nfn work() {}\n#[cfg(unix)] struct Missing;\n#[cfg(not(unix))]\n/// `Missing` is unavailable in this scope.\nfn other() {}",
        ),
        &[2, 4, 5],
    );
}

#[test]
fn given_single_field_owner_in_another_file_when_checked_then_data_and_enum_findings_use_declaration_anchors()
 {
    let context = read_shape_context(
        "mod model { mod owner; mod payload; pub use payload::Payload; pub use owner::Owner; }",
        &[
            (
                "model/owner.rs",
                "pub struct Owner { value: super::Payload }",
            ),
            ("model/payload.rs", "pub struct Payload { value: u64 }"),
        ],
    );
    assert_lines(&NestedDataStruct, &context, &[1]);
    let findings = NestedDataStruct.check(&context);
    assert_eq!(
        findings[0].finding.location.path,
        "modules/sample/src/model/payload.rs"
    );
    let context = read_shape_context(
        "mod model { mod owner; mod status; pub use status::Status; pub use owner::Owner; }",
        &[
            (
                "model/owner.rs",
                "pub struct Owner { status: super::Status }",
            ),
            ("model/status.rs", "pub enum Status { Ready, Done }"),
        ],
    );
    assert_lines(&NestedEnum, &context, &[1]);
}

#[test]
fn given_shared_users_aliases_and_colocated_types_when_checked_then_exclusivity_is_not_guessed() {
    let context = read_shape_context(
        "mod model { mod owner; mod payload; type Alias = payload::Payload; pub use owner::Owner; }",
        &[
            ("model/owner.rs", "pub struct Owner { value: super::Alias }"),
            ("model/payload.rs", "pub struct Payload { value: u64 }"),
        ],
    );
    assert_lines(&NestedDataStruct, &context, &[1]);
    let context = read_shape_context(
        "mod model { mod owner; mod payload; mod other; pub use payload::Payload; }",
        &[
            (
                "model/owner.rs",
                "pub struct Owner { value: super::Payload }",
            ),
            (
                "model/other.rs",
                "pub struct Other { value: super::Payload }",
            ),
            ("model/payload.rs", "pub struct Payload { value: u64 }"),
        ],
    );
    assert_lines(&NestedDataStruct, &context, &[]);
    let context = read_shape_context(
        "mod model { mod owner; }",
        &[(
            "model/owner.rs",
            "pub struct Owner { value: Payload } struct Payload { value: u64 } impl Owner { fn payload(&self) -> &Payload { &self.value } }",
        )],
    );
    assert_lines(&NestedDataStruct, &context, &[]);
}

#[test]
fn given_proven_associated_factory_consumer_when_checked_then_returned_data_requires_its_own_model_file()
 {
    let context = read_shape_context(
        "mod model { mod owner; mod consumer; pub use owner::Owner; }",
        &[
            (
                "model/owner.rs",
                "pub struct Owner; pub struct Payload { value: u64 } impl Owner { pub fn make() -> Payload { Payload { value: 1 } } }",
            ),
            (
                "model/consumer.rs",
                "pub struct Consumer; impl Consumer { fn new() -> Self { let value = super::Owner::make(); Self } }",
            ),
        ],
    );
    assert_lines(&NestedDataStruct, &context, &[1]);
    let context = read_shape_context(
        "mod model { mod owner; mod consumer; mod payload; pub use owner::Owner; pub use payload::Payload; }",
        &[
            (
                "model/owner.rs",
                "pub struct Owner; impl Owner { pub fn make() -> super::Payload { super::Payload { value: 1 } } }",
            ),
            (
                "model/consumer.rs",
                "pub struct Consumer; impl Consumer { fn new() -> Self { let value = super::Owner::make(); Self } }",
            ),
            ("model/payload.rs", "pub struct Payload { pub value: u64 }"),
        ],
    );
    assert_lines(&NestedDataStruct, &context, &[]);
}

#[test]
fn given_returned_io_adapter_when_checked_then_does_not_classify_it_as_plain_data() {
    let context = read_shape_context(
        "mod file { mod adapter; mod owner; mod consumer; pub use adapter::FileStore; pub use owner::Owner; }",
        &[
            (
                "file/adapter.rs",
                "pub struct FileStore { path: String } impl FileStore { pub fn read(&self) -> String { std::fs::read_to_string(&self.path).unwrap() } }",
            ),
            (
                "file/owner.rs",
                "pub struct Owner; impl Owner { pub fn store() -> super::FileStore { todo!() } }",
            ),
            (
                "file/consumer.rs",
                "pub struct Consumer; impl Consumer { fn new() -> Self { let store = super::Owner::store(); Self } }",
            ),
        ],
    );
    assert_lines(&NestedDataStruct, &context, &[]);
}
