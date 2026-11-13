use crate::check::api::Rule;
use crate::check::model::Anchor;
use crate::check::model::Finding;
use crate::check::model::Location;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::model::RuleId;
use crate::comment::rule::DocCommentVisitor;
use crate::comment::util::find_forbidden_shape;
use crate::source::model::Comment;
use crate::source::model::CommentKind;
use crate::source::model::SourceFile;

pub(crate) struct ForbiddenComment;

impl ForbiddenComment {
    fn strip_opener(comment: &Comment) -> &str {
        match comment.kind {
            CommentKind::Line => comment.text.strip_prefix("//").unwrap_or(&comment.text),
            CommentKind::Block => {
                let inner = comment.text.strip_prefix("/*").unwrap_or(&comment.text);
                inner.strip_suffix("*/").unwrap_or(inner)
            }
        }
    }

    fn check_text(file: &SourceFile, line: u32, text: &str, is_doc: bool) -> Option<RuleFinding> {
        let shape = find_forbidden_shape(text, is_doc)?;
        Some(RuleFinding {
            finding: Finding {
                rule_id: RuleId(Self::RULE.to_owned()),
                location: Location {
                    path: file.path.clone(),
                    line: Some(line),
                },
                subject: file.path.clone(),
                message: format!(
                    "`{}` is {shape}: region.md allows none in code written by hand",
                    text.trim()
                ),
            },
            anchor: Anchor::Unexpectable,
        })
    }

    fn strip_gutter<'text>(comment: &Comment, line_text: &'text str) -> &'text str {
        if !matches!(comment.kind, CommentKind::Block) {
            return line_text;
        }
        match line_text.trim_start().strip_prefix('*') {
            Some(rest) if !rest.starts_with('*') => rest,
            _ => line_text,
        }
    }

    fn check_comment(
        file: &SourceFile,
        comment: &Comment,
        text: &str,
        is_doc: bool,
    ) -> Vec<RuleFinding> {
        text.lines()
            .zip(0_u32..)
            .filter_map(|(line_text, offset)| {
                Self::check_text(
                    file,
                    comment.start.line.saturating_add(offset),
                    Self::strip_gutter(comment, line_text),
                    is_doc,
                )
            })
            .collect()
    }

    fn check_file(file: &SourceFile) -> Vec<RuleFinding> {
        let mut findings: Vec<RuleFinding> = file
            .comments
            .iter()
            .flat_map(|comment| {
                Self::check_comment(file, comment, Self::strip_opener(comment), false)
            })
            .collect();
        let doc_comments = DocCommentVisitor::new().collect_comments(&file.syntax);
        findings.extend(
            doc_comments
                .iter()
                .flat_map(|comment| Self::check_comment(file, comment, &comment.text, true)),
        );
        findings
    }

    const RULE: &str = "ForbiddenComment";
}

impl Rule for ForbiddenComment {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .crates
            .iter()
            .flat_map(|sources| &sources.files)
            .filter(|file| !file.is_generated)
            .flat_map(Self::check_file)
            .collect()
    }
}
