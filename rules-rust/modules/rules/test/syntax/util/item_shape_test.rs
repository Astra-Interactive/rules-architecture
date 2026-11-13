use crate::check::api::Rule;
use crate::test::check::usecase::read_case_context;

#[test]
fn given_non_path_self_type_and_union_declaration_when_read_then_distinguishes_them() {
    let self_type: syn::Type = syn::parse_str("(usize, usize)").expect("valid type");
    assert_eq!(None, crate::syntax::util::find_self_name(&self_type));
    let item: syn::Item = syn::parse_str("union Value { bits: usize }").expect("valid union");
    assert_eq!(
        "Value",
        crate::syntax::util::find_type_name(&item)
            .expect("union name")
            .to_string()
    );
}

#[test]
fn given_constructor_signatures_when_classified_then_returns_only_associated_constructors() {
    for source in [
        "fn new() -> Self",
        "fn with_value(value: usize) -> Self",
        "fn from_value(value: usize) -> Self",
        "fn open() -> Option<Self>",
        "fn open() -> Result<Subject, Error>",
        "fn open() -> Subject",
    ] {
        let signature = syn::parse_str::<syn::Signature>(source).expect("valid signature");
        assert!(crate::syntax::util::is_constructor(&signature, "Subject"));
    }
    for source in [
        "fn read(&self) -> Self",
        "fn read() -> usize",
        "fn read()",
        "fn read() -> &Subject",
        "fn read() -> Result",
        "fn read() -> Result<5>",
    ] {
        let signature = syn::parse_str::<syn::Signature>(source).expect("valid signature");
        assert!(!crate::syntax::util::is_constructor(&signature, "Subject"));
    }
}

#[test]
fn given_acronyms_when_snake_cased_then_preserves_word_boundaries() {
    assert_eq!(
        "http_client",
        crate::syntax::util::to_snake_case("HTTPClient")
    );
    assert_eq!(
        "sha256_digest",
        crate::syntax::util::to_snake_case("Sha256Digest")
    );
    assert_eq!("", crate::syntax::util::to_snake_case(""));
}

pub(crate) fn assert_syntax_findings(
    rule: &dyn Rule,
    case_rule: &str,
    source: &str,
    expected: usize,
) {
    let mut context = read_case_context(case_rule, "pass-syntax");
    let file = context.crates[0]
        .files
        .iter_mut()
        .find(|file| !file.module.path.is_empty())
        .expect("fixture has a source file");
    file.syntax = syn::parse_file(source).expect("test input parses");
    file.comments = crate::source::util::scan_comments(source, &[]);
    let findings = rule.check(&context);
    assert_eq!(
        expected,
        findings.len(),
        "source: {source}\nfindings: {findings:?}"
    );
}
