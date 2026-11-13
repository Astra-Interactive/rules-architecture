use crate::gate::model::DocumentBlock;
use crate::gate::model::ExampleEntry;
use crate::gate::model::ExampleFragment;
use crate::gate::model::SyncChange;
use crate::gate::util::compute_example_hash;
use crate::gate::util::sync_examples;
use std::collections::BTreeSet;

fn build_block(first_line: &str, lines: &[&str]) -> DocumentBlock {
    DocumentBlock {
        document: "doc.rs.md".to_owned(),
        heading: "# Rule".to_owned(),
        first_line: first_line.to_owned(),
        lines: lines.iter().map(|line| (*line).to_owned()).collect(),
    }
}

fn build_entry(first_line: &str, first: usize, last: usize, sha256: &str) -> ExampleEntry {
    ExampleEntry {
        document: "doc.rs.md".to_owned(),
        anchor: "# Rule".to_owned(),
        first_line: first_line.to_owned(),
        occurrence: 1,
        wrap: "fn-body".to_owned(),
        sha256: sha256.to_owned(),
        fragments: vec![ExampleFragment {
            first_block_line: first,
            last_block_line: last,
            suffix: String::new(),
            translations: Vec::new(),
        }],
    }
}

#[test]
fn given_documents_and_entries_when_synced_then_reports_changed_moved_unplaced_and_new_blocks() {
    let placed = build_block(
        "// Bad",
        &["// Bad", "bad();", "", "// Good", "good();", "extra();"],
    );
    let unknown = build_block("fn other() {}", &["fn other() {}"]);
    let documents = BTreeSet::from(["doc.rs.md".to_owned()]);
    let current = build_entry("// Bad", 2, 2, "");
    let current_hash =
        compute_example_hash(&current, &placed).expect("the fragment lies in the block");
    let entries = vec![
        build_entry("// Bad", 2, 2, &current_hash),
        build_entry("// Bad", 5, 5, "stale"),
        build_entry("fn gone() {}", 1, 1, "kept"),
    ];

    let result = sync_examples(&entries, &[placed, unknown], &documents);

    assert_eq!(
        vec![
            SyncChange::Changed {
                document: "doc.rs.md".to_owned(),
                first_line: "// Bad".to_owned(),
            },
            SyncChange::Moved {
                document: "doc.rs.md".to_owned(),
                first_line: "fn gone() {}".to_owned(),
            },
            SyncChange::Unplaced {
                document: "doc.rs.md".to_owned(),
                heading: "# Rule".to_owned(),
                line: "extra();".to_owned(),
            },
            SyncChange::New {
                document: "doc.rs.md".to_owned(),
                heading: "# Rule".to_owned(),
                first_line: "fn other() {}".to_owned(),
            },
        ],
        result.changes
    );
    assert_eq!("kept", result.hashes[2]);
}

#[test]
fn given_entry_that_takes_the_whole_block_when_synced_then_no_line_is_unplaced() {
    let block = build_block("note", &["note", "more"]);
    let mut entry = build_entry("note", 1, 1, "");
    entry.wrap = "none".to_owned();
    entry.sha256 = compute_example_hash(&entry, &block).expect("the fragment lies in the block");

    let result = sync_examples(
        &[entry],
        &[block],
        &BTreeSet::from(["doc.rs.md".to_owned()]),
    );

    assert!(result.changes.is_empty());
}
