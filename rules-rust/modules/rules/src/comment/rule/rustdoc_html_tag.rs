use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::comment::rule::DocCommentVisitor;
use crate::comment::util::strip_doc_code;
use crate::source::model::Comment;
use crate::source::model::NodeSpan;
use crate::source::model::SourceFile;

pub(crate) struct RustdocHtmlTag;

impl RustdocHtmlTag {
    fn has_tag(text: &str) -> bool {
        text.match_indices('<').any(|(start, _)| {
            let rest = text[start + 1..]
                .strip_prefix('/')
                .unwrap_or(&text[start + 1..]);
            let name_length = rest.bytes().take_while(u8::is_ascii_alphanumeric).count();
            if name_length == 0 || !rest.as_bytes()[0].is_ascii_alphabetic() {
                return false;
            }
            let tail = &rest[name_length..];
            let is_delimiter = tail.starts_with('>')
                || tail.starts_with('/')
                || tail.starts_with(char::is_whitespace);
            is_delimiter && tail.contains('>')
        })
    }

    fn check_comments(file: &SourceFile, comments: &[Comment]) -> Vec<RuleFinding> {
        let text = comments
            .iter()
            .map(|comment| comment.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let cleaned = strip_doc_code(&text);
        let mut findings = Vec::new();
        let mut offset = 0;
        for comment in comments {
            let line_count = comment.text.lines().count().max(1);
            let section = cleaned
                .lines()
                .skip(offset)
                .take(line_count)
                .collect::<Vec<_>>()
                .join("\n");
            if Self::has_tag(&section) {
                findings.push(build_node_finding(
                    "RustdocHtmlTag",
                    &file.path,
                    NodeSpan {
                        start: comment.start,
                        end: comment.start,
                    },
                    file.path.clone(),
                    "Use Markdown instead of HTML in rustdoc (rustdoc.rs.md)".to_owned(),
                ));
            }
            offset += line_count;
        }
        findings
    }
}

impl Rule for RustdocHtmlTag {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for file in context.crates.iter().flat_map(|sources| &sources.files) {
            let comments = DocCommentVisitor::new().collect_comments(&file.syntax);
            let mut first = 0;
            while first < comments.len() {
                let mut end = first + 1;
                while end < comments.len()
                    && comments[end].start.line
                        <= comments[end - 1].start.line.saturating_add(
                            u32::try_from(comments[end - 1].text.lines().count())
                                .unwrap_or(u32::MAX)
                                .max(1),
                        )
                {
                    end += 1;
                }
                findings.extend(Self::check_comments(file, &comments[first..end]));
                first = end;
            }
        }
        findings
    }
}
