use crate::source::model::Comment;
use crate::source::model::TextPosition;
use crate::source::util::find_node_span;
use crate::source::util::parse_macro_arguments;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct ImportOrderVisitor<'source> {
    comments: &'source [Comment],
    check_modules: bool,
    findings: Vec<proc_macro2::Span>,
}

impl<'source> ImportOrderVisitor<'source> {
    pub(crate) fn new(comments: &'source [Comment], check_modules: bool) -> Self {
        Self {
            comments,
            check_modules,
            findings: Vec::new(),
        }
    }

    fn has_comment_between(&self, previous: TextPosition, current: TextPosition) -> bool {
        self.comments.iter().any(|comment| {
            let position = comment.start;
            let after_previous = position.line > previous.line
                || (position.line == previous.line && position.column >= previous.column);
            let before_current = position.line < current.line
                || (position.line == current.line && position.column < current.column);
            after_previous && before_current
        })
    }

    fn check_items(&mut self, items: &[syn::Item], is_file: bool) {
        let mut has_item = false;
        let mut previous_end: Option<TextPosition> = None;
        for item in items {
            if let syn::Item::Use(value) = item {
                let span = find_node_span(value.span());
                let has_gap = is_file
                    && previous_end.is_some_and(|previous| {
                        span.start.line > previous.line + 1
                            || self.has_comment_between(previous, span.start)
                    });
                if has_item || has_gap {
                    self.findings.push(value.span());
                }
                previous_end = Some(span.end);
            } else {
                has_item = true;
            }
        }
    }

    pub(crate) fn collect_findings(mut self, file: &syn::File) -> Vec<proc_macro2::Span> {
        self.check_items(&file.items, true);
        self.visit_file(file);
        self.findings
    }
}

impl<'ast> Visit<'ast> for ImportOrderVisitor<'_> {
    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        for argument in &parse_macro_arguments(item) {
            self.visit_expr(argument);
        }
    }

    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        if self.check_modules
            && let Some((_, items)) = &item.content
        {
            self.check_items(items, false);
        }
        syn::visit::visit_item_mod(self, item);
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        let mut has_statement = false;
        for statement in &block.stmts {
            if let syn::Stmt::Item(syn::Item::Use(value)) = statement {
                if has_statement {
                    self.findings.push(value.span());
                }
            } else {
                has_statement = true;
            }
        }
        syn::visit::visit_block(self, block);
    }
}
