use crate::source::model::Comment;
use crate::source::model::CommentKind;
use crate::source::model::TextPosition;
use std::ops::Range;

fn find_text_position(text: &str, offset: usize) -> TextPosition {
    let before = &text[..offset.min(text.len())];
    let line_start = before.rfind('\n').map_or(0, |newline| newline + 1);
    let line = before.matches('\n').count().saturating_add(1);
    let column = before[line_start..].chars().count();
    TextPosition {
        line: u32::try_from(line).unwrap_or(u32::MAX),
        column: u32::try_from(column).unwrap_or(u32::MAX),
    }
}

fn find_block_comment_end(gap: &str) -> usize {
    let bytes = gap.as_bytes();
    let mut depth = 0_usize;
    let mut index = 0;
    while index + 1 < bytes.len() {
        let pair = &bytes[index..index + 2];
        if pair == b"/*" {
            depth += 1;
            index += 2;
        } else if pair == b"*/" {
            depth = depth.saturating_sub(1);
            index += 2;
            if depth == 0 {
                return index;
            }
        } else {
            index += 1;
        }
    }
    bytes.len()
}

fn scan_gap(text: &str, gap: Range<usize>, comments: &mut Vec<Comment>) {
    let mut cursor = gap.start;
    while cursor < gap.end {
        let rest = &text[cursor..gap.end];
        let Some(slash) = rest.find('/') else {
            return;
        };
        let start = cursor + slash;
        let candidate = &text[start..gap.end];
        let length = if candidate.starts_with("//") {
            candidate.find('\n').unwrap_or(candidate.len())
        } else if candidate.starts_with("/*") {
            find_block_comment_end(candidate)
        } else {
            cursor = start + 1;
            continue;
        };
        let kind = if candidate.starts_with("//") {
            CommentKind::Line
        } else {
            CommentKind::Block
        };
        comments.push(Comment {
            start: find_text_position(text, start),
            text: candidate[..length].to_owned(),
            kind,
        });
        cursor = start + length;
    }
}

pub(crate) fn scan_comments(text: &str, covered: &[Range<usize>]) -> Vec<Comment> {
    let mut sorted: Vec<Range<usize>> = covered.to_vec();
    sorted.sort_by_key(|range| range.start);
    let mut comments = Vec::new();
    let mut cursor = 0;
    for range in sorted {
        if range.start > cursor {
            scan_gap(text, cursor..range.start, &mut comments);
        }
        cursor = cursor.max(range.end);
    }
    if cursor < text.len() {
        scan_gap(text, cursor..text.len(), &mut comments);
    }
    comments
}
