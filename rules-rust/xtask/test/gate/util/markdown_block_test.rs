use crate::gate::model::DocumentBlock;
use crate::gate::util::extract_blocks;

#[test]
fn given_indented_fences_under_headings_when_extracted_then_blocks_carry_heading_and_dedented_lines()
 {
    let text = "# Rule\n\n- item:\n\n  ```rust\n  fn first() {}\n      nested();\n  ```\n\n## Second\n```toml\n[a]\n```\n";

    let blocks = extract_blocks("doc.rs.md", text);

    assert_eq!(
        vec![
            DocumentBlock {
                document: "doc.rs.md".to_owned(),
                heading: "# Rule".to_owned(),
                first_line: "fn first() {}".to_owned(),
                lines: vec!["fn first() {}".to_owned(), "    nested();".to_owned()],
            },
            DocumentBlock {
                document: "doc.rs.md".to_owned(),
                heading: "## Second".to_owned(),
                first_line: "[a]".to_owned(),
                lines: vec!["[a]".to_owned()],
            },
        ],
        blocks
    );
}

#[test]
fn given_tilde_and_longer_fences_when_extracted_then_only_matching_complete_closers_end_the_block()
{
    let text = "# Rule\n~~~~rust\nfirst\n```\n~~~\n~~~~ nope\n~~~~~\n````rust\nsecond\n```\n````\n~~~rust\nunterminated\n";
    let blocks = extract_blocks("rule.rs.md", text);
    assert_eq!(blocks.len(), 3);
    assert_eq!(blocks[0].lines, ["first", "```", "~~~", "~~~~ nope"]);
    assert_eq!(blocks[1].lines, ["second", "```"]);
    assert_eq!(blocks[2].lines, ["unterminated"]);
    assert!(extract_blocks("rule.rs.md", "``rust\n```bad`info\n").is_empty());
    let unicode = extract_blocks("rule.rs.md", "  ```rust\n\u{2003}let value = 1;\n  ```\n");
    assert_eq!(unicode[0].lines, ["\u{2003}let value = 1;"]);
}
