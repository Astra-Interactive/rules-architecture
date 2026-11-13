use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::comment::rule::DocCommentVisitor;
use crate::comment::util::strip_doc_code;
use crate::readme::util::count_sentences;
use crate::source::model::NodeSpan;

pub(crate) struct RustdocFiller;

impl RustdocFiller {
    fn is_filler(text: &str) -> bool {
        let normalized = text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        let is_kind_filler = Self::ITEM_KINDS.iter().any(|kind| {
            Self::FILLER_PREDICATES
                .iter()
                .any(|predicate| normalized.contains(&format!("this {kind} {predicate}")))
        });
        let is_result_filler = Self::RESULT_PHRASES
            .iter()
            .any(|phrase| normalized.contains(phrase));
        if is_kind_filler || is_result_filler {
            return true;
        }
        normalized
            .split(['.', '!', '?'])
            .any(|sentence| sentence.trim() == "returns the result")
    }

    const FILLER_PREDICATES: [&str; 3] = ["is responsible for", "allows you to", "is intended for"];
    const ITEM_KINDS: [&str; 8] = [
        "function", "method", "struct", "enum", "trait", "type", "module", "crate",
    ];
    const RESULT_PHRASES: [&str; 2] = [
        "returns the result of the operation",
        "returns the returned value",
    ];
}

impl Rule for RustdocFiller {
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
                let text = comments[first..end]
                    .iter()
                    .map(|comment| comment.text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                let cleaned = strip_doc_code(&text);
                let summary = cleaned
                    .trim_start()
                    .lines()
                    .take_while(|line| {
                        !line.trim().is_empty() && !line.trim_start().starts_with('#')
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                if Self::is_filler(&cleaned) || count_sentences(&summary) > 4 {
                    let start = comments[first].start;
                    findings.push(build_node_finding("RustdocFiller", &file.path, NodeSpan { start, end: start }, file.path.clone(), "Document information absent from the signature without filler, in at most four summary sentences (rustdoc.rs.md)".to_owned()));
                }
                first = end;
            }
            for span in crate::comment::rule::RustdocParameterVisitor::collect(&file.syntax) {
                findings.push(build_node_finding("RustdocFiller", &file.path, span, file.path.clone(), "parameter documentation adds constraints or semantics instead of repeating the parameter's name".to_owned()));
            }
        }
        findings
    }
}
