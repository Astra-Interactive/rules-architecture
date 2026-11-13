use crate::source::model::Comment;
use crate::source::model::CommentKind;
use crate::source::model::TextPosition;
use crate::source::util::scan_comments;

#[test]
fn given_gaps_between_tokens_when_scanned_then_returns_line_nested_and_unterminated_comments() {
    let text = "a // line\nb /* outer /* inner */ */ c /* open";
    let covered = vec![0..1, 10..11, 36..37];

    let comments = scan_comments(text, &covered);

    assert_eq!(
        vec![
            Comment {
                start: TextPosition { line: 1, column: 2 },
                text: "// line".to_owned(),
                kind: CommentKind::Line,
            },
            Comment {
                start: TextPosition { line: 2, column: 2 },
                text: "/* outer /* inner */ */".to_owned(),
                kind: CommentKind::Block,
            },
            Comment {
                start: TextPosition {
                    line: 2,
                    column: 28
                },
                text: "/* open".to_owned(),
                kind: CommentKind::Block,
            },
        ],
        comments
    );
}

#[test]
fn given_a_slash_that_opens_no_comment_when_scanned_then_returns_nothing() {
    let comments = scan_comments("a / b", &[0..1, 4..5]);

    assert!(comments.is_empty());
}
