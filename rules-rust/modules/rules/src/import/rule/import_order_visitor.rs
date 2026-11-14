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

    fn has_trailing_comment(&self, end: TextPosition, next: Option<TextPosition>) -> bool {
        self.comments.iter().any(|comment| {
            let is_on_line = comment.start.line == end.line && comment.start.column >= end.column;
            let is_before_next = next
                .is_none_or(|token| token.line != end.line || comment.start.column < token.column);
            is_on_line && is_before_next
        })
    }

    fn push_finding(&mut self, span: proc_macro2::Span) {
        let anchor = find_node_span(span);
        let is_known = self
            .findings
            .iter()
            .any(|known| find_node_span(*known) == anchor);
        if !is_known {
            self.findings.push(span);
        }
    }

    fn has_gap_after(&self, last_use: Option<&syn::ItemUse>, value: &syn::ItemUse) -> bool {
        let span = find_node_span(value.span());
        last_use.is_some_and(|last| {
            let previous = find_node_span(last.span()).end;
            span.start.line > previous.line + 1 || self.has_comment_between(previous, span.start)
        })
    }

    fn check_last_use(&mut self, last_use: Option<&syn::ItemUse>, next: Option<TextPosition>) {
        if let Some(last) = last_use
            && self.has_trailing_comment(find_node_span(last.span()).end, next)
        {
            self.push_finding(last.span());
        }
    }

    fn check_items(
        &mut self,
        items: &[syn::Item],
        is_ordered: bool,
        closing: Option<TextPosition>,
    ) {
        let mut has_item = false;
        let mut last_use: Option<&syn::ItemUse> = None;
        let mut next: Option<TextPosition> = closing;
        let is_crate_root = closing.is_none() && !self.check_modules;
        for (position, item) in items.iter().enumerate() {
            if let syn::Item::Use(value) = item {
                let is_re_export = !matches!(value.vis, syn::Visibility::Inherited);
                if is_crate_root && is_re_export {
                    continue;
                }
                let has_gap = self.has_gap_after(last_use, value);
                if (is_ordered && has_item) || has_gap {
                    self.push_finding(value.span());
                }
                last_use = Some(value);
                next = items
                    .get(position + 1)
                    .map(|following| find_node_span(following.span()).start)
                    .or(closing);
            } else {
                has_item = true;
            }
        }
        self.check_last_use(last_use, next);
    }

    pub(crate) fn collect_findings(mut self, file: &syn::File) -> Vec<proc_macro2::Span> {
        self.check_items(&file.items, true, None);
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
        if let Some((brace, items)) = &item.content {
            let closing = find_node_span(brace.span.close()).start;
            self.check_items(items, self.check_modules, Some(closing));
        }
        syn::visit::visit_item_mod(self, item);
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        let mut has_statement = false;
        let mut last_leading_use: Option<&syn::ItemUse> = None;
        let mut last_leading_position = 0;
        for (position, statement) in block.stmts.iter().enumerate() {
            if let syn::Stmt::Item(syn::Item::Use(value)) = statement {
                let is_misplaced = has_statement || self.has_gap_after(last_leading_use, value);
                if is_misplaced {
                    self.push_finding(value.span());
                }
                if !has_statement {
                    last_leading_use = Some(value);
                    last_leading_position = position;
                }
                continue;
            }
            has_statement = true;
        }
        let closing = find_node_span(block.brace_token.span.close()).start;
        let next = block
            .stmts
            .get(last_leading_position + 1)
            .map_or(closing, |following| find_node_span(following.span()).start);
        self.check_last_use(last_leading_use, Some(next));
        syn::visit::visit_block(self, block);
    }
}
