use crate::gate::model::ExampleFragment;
use crate::gate::model::GateError;
use crate::gate::model::Translation;
use crate::gate::util::parse_examples;

#[test]
fn given_examples_file_when_parsed_then_reads_commit_entries_and_fragments() {
    let text = r##"
[sync]
conf-ide-commit = "be1dbbf"

[[example]]
document = "doc.rs.md"
anchor = "# Rule"
first-line = "// Bad"
occurrence = 2
wrap = "fn-body"
sha256 = "abc"
[[example.fragment]]
block-lines = "2-3"
suffix = ";"
translate = [{ from = "a", to = "b" }]

[[example]]
document = "doc.rs.md"
anchor = "# Rule"
first-line = "fn first() {}"
wrap = "none"
sha256 = ""
"##;

    let examples = parse_examples(text).expect("the examples parse");

    assert_eq!("be1dbbf", examples.commit);
    assert_eq!(2, examples.entries[0].occurrence);
    assert_eq!(1, examples.entries[1].occurrence);
    assert_eq!(
        vec![ExampleFragment {
            first_block_line: 2,
            last_block_line: 3,
            suffix: ";".to_owned(),
            translations: vec![Translation {
                from: "a".to_owned(),
                to: "b".to_owned(),
            }],
        }],
        examples.entries[0].fragments
    );
    assert!(examples.entries[1].fragments.is_empty());
}

#[test]
fn given_malformed_block_lines_when_parsed_then_returns_examples_error() {
    let text = "[[example]]\ndocument = \"d\"\nanchor = \"a\"\nfirst-line = \"f\"\nwrap = \"w\"\nsha256 = \"\"\n[[example.fragment]]\nblock-lines = \"two\"\n";

    assert!(matches!(
        parse_examples(text),
        Err(GateError::Examples { .. })
    ));
}

#[test]
fn given_missing_document_when_parsed_then_returns_examples_error() {
    assert!(matches!(
        parse_examples("[[example]]\nanchor = \"a\"\n"),
        Err(GateError::Examples { .. })
    ));
}

#[test]
fn given_text_that_is_not_toml_when_parsed_then_returns_examples_error() {
    assert!(matches!(
        parse_examples("[[example"),
        Err(GateError::Examples { .. })
    ));
}

#[test]
fn given_invalid_numeric_ranges_and_translations_when_parsed_then_reports_the_invalid_entry() {
    for fragment in [
        "block-lines=\"a-2\"",
        "block-lines=\"1-b\"",
        "block-lines=\"1-2\"\ntranslate=[{from=\"x\"}]",
    ] {
        let text = format!(
            "[[example]]\ndocument=\"d\"\nanchor=\"a\"\nfirst-line=\"f\"\nwrap=\"w\"\nsha256=\"\"\n[[example.fragment]]\n{fragment}\n"
        );
        assert!(matches!(
            parse_examples(&text),
            Err(GateError::Examples { .. })
        ));
    }
}
