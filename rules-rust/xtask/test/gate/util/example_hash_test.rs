use crate::gate::model::DocumentBlock;
use crate::gate::model::ExampleEntry;
use crate::gate::model::ExampleFragment;
use crate::gate::model::Translation;
use crate::gate::util::compute_example_hash;
use crate::gate::util::is_marker_line;

fn build_block() -> DocumentBlock {
    DocumentBlock {
        document: "doc.rs.md".to_owned(),
        heading: "# Rule".to_owned(),
        first_line: "// Bad".to_owned(),
        lines: vec![
            "// Bad".to_owned(),
            "fn load(&self) -> Order { /* ... */ }".to_owned(),
            "".to_owned(),
            "// Good".to_owned(),
            "fn load(&self) -> Result<Order, OrderError>".to_owned(),
        ],
    }
}

fn build_entry(fragments: Vec<ExampleFragment>) -> ExampleEntry {
    ExampleEntry {
        document: "doc.rs.md".to_owned(),
        anchor: "# Rule".to_owned(),
        first_line: "// Bad".to_owned(),
        occurrence: 1,
        wrap: "fn-body".to_owned(),
        sha256: String::new(),
        fragments,
    }
}

#[test]
fn given_marker_comments_when_checked_then_only_bad_good_deleted_and_kept_are_markers() {
    assert!(is_marker_line("  // Bad: two writes"));
    assert!(is_marker_line("// Kept: the signature"));
    assert!(!is_marker_line("// modules/billing/api/src/lib.rs"));
    assert!(!is_marker_line("let bad = 1;"));
}

#[test]
fn given_fragment_with_translation_and_suffix_when_hashed_then_equals_the_rendered_lines() {
    let fragment = ExampleFragment {
        first_block_line: 5,
        last_block_line: 5,
        suffix: " {".to_owned(),
        translations: vec![Translation {
            from: "OrderError".to_owned(),
            to: "LoadError".to_owned(),
        }],
    };

    let hash = compute_example_hash(&build_entry(vec![fragment]), &build_block());

    let rendered = DocumentBlock {
        lines: vec![
            "  fn load(&self) -> Result<Order, LoadError> {".to_owned(),
            "".to_owned(),
        ],
        ..build_block()
    };
    assert_eq!(
        compute_example_hash(&build_entry(Vec::new()), &rendered),
        hash
    );
}

#[test]
fn given_blocks_that_differ_in_blank_lines_and_indentation_when_hashed_then_hashes_are_equal() {
    let block = build_block();
    let reformatted = DocumentBlock {
        lines: block
            .lines
            .iter()
            .map(|line| format!("    {line}"))
            .filter(|line| !line.trim().is_empty())
            .collect(),
        ..build_block()
    };

    let hash = compute_example_hash(&build_entry(Vec::new()), &block);

    assert_eq!(
        compute_example_hash(&build_entry(Vec::new()), &reformatted),
        hash
    );
}

#[test]
fn given_fragment_beyond_the_block_when_hashed_then_returns_none() {
    let fragment = ExampleFragment {
        first_block_line: 4,
        last_block_line: 9,
        suffix: String::new(),
        translations: Vec::new(),
    };

    assert_eq!(
        None,
        compute_example_hash(&build_entry(vec![fragment]), &build_block())
    );
}
