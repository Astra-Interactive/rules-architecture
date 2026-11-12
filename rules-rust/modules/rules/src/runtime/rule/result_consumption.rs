use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::{build_node_finding, format_module_path};
use crate::runtime::rule::{FallibleStepVisitor, TypedRuntimeVisitor};
use crate::source::model::NodeSpan;
use crate::source::model::SourceFile;
use crate::source::util::find_node_span;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct ResultConsumption;

struct ConsumptionVisitor<'source> {
    file: &'source SourceFile,
    allowed: bool,
    spans: Vec<proc_macro2::Span>,
    composed_steps: Vec<proc_macro2::Span>,
}

impl ConsumptionVisitor<'_> {
    fn check_composed(&mut self, expression: &syn::Expr) {
        if FallibleStepVisitor::count(expression) > 1 {
            self.composed_steps.push(expression.span());
        }
    }

    fn check_expression(&mut self, expression: &syn::Expr) {
        match expression {
            syn::Expr::Block(block) => self.check_tail(&block.block),
            syn::Expr::Unsafe(block) => self.check_tail(&block.block),
            syn::Expr::If(branch) => {
                self.check_composed(&branch.cond);
                self.check_tail(&branch.then_branch);
                if let Some((_, otherwise)) = &branch.else_branch {
                    self.check_expression(otherwise);
                }
            }
            syn::Expr::Match(selection) => {
                self.check_composed(&selection.expr);
                for arm in &selection.arms {
                    self.check_expression(&arm.body);
                }
            }
            syn::Expr::Loop(repeat) => self.check_tail(&repeat.body),
            syn::Expr::While(repeat) => {
                self.check_composed(&repeat.cond);
                self.check_tail(&repeat.body);
            }
            syn::Expr::ForLoop(repeat) => {
                self.check_composed(&repeat.expr);
                self.check_tail(&repeat.body);
            }
            _ => self.check_composed(expression),
        }
    }

    fn check_tail(&mut self, block: &syn::Block) {
        if let Some(syn::Stmt::Expr(tail, None)) = block.stmts.last() {
            self.check_expression(tail);
        }
    }

    fn check_statements(&mut self, block: &syn::Block) {
        let last = block.stmts.len().saturating_sub(1);
        for (position, statement) in block.stmts.iter().enumerate() {
            if let syn::Stmt::Expr(expression, semicolon) = statement
                && (semicolon.is_some() || position != last)
            {
                self.check_expression(expression);
            }
        }
    }

    fn list_outermost_steps(&self) -> Vec<NodeSpan> {
        let spans: Vec<NodeSpan> = self
            .composed_steps
            .iter()
            .map(|span| find_node_span(*span))
            .collect();
        spans
            .iter()
            .filter(|span| {
                !spans
                    .iter()
                    .any(|outer| outer != *span && outer.contains(span))
            })
            .copied()
            .collect()
    }
}

fn expects_discard(attribute: &syn::Attribute) -> bool {
    if !attribute.path().is_ident("expect") {
        return false;
    }
    let Ok(entries) = attribute.parse_args_with(
        syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
    ) else {
        return false;
    };
    entries.iter().any(|entry| {
        matches!(entry, syn::Meta::Path(path)
        if path.segments.len() == 2 && path.segments[0].ident == "clippy"
        && path.segments[1].ident == "let_underscore_must_use")
    })
}

impl<'ast> Visit<'ast> for ConsumptionVisitor<'_> {
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        self.check_tail(&item.block);
        syn::visit::visit_item_fn(self, item);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        self.check_tail(&item.block);
        syn::visit::visit_impl_item_fn(self, item);
    }

    fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure) {
        self.check_expression(&closure.body);
        syn::visit::visit_expr_closure(self, closure);
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        self.check_statements(block);
        syn::visit::visit_block(self, block);
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        let parent = self.allowed;
        self.allowed = matches!(local.pat, syn::Pat::Wild(_));
        for attribute in &local.attrs {
            self.visit_attribute(attribute);
        }
        self.allowed = false;
        if let Some(init) = &local.init {
            if FallibleStepVisitor::count(&init.expr) > 1 {
                self.composed_steps.push(local.span());
            }
            self.visit_expr(&init.expr);
            if let Some((_, diverge)) = &init.diverge {
                self.visit_expr(diverge);
            }
        }
        self.allowed = parent;
    }

    fn visit_attribute(&mut self, attribute: &'ast syn::Attribute) {
        if !self.allowed && expects_discard(attribute) {
            self.spans.push(attribute.span());
        }
    }
}

impl Rule for ResultConsumption {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for sources in &context.crates {
            for file in &sources.files {
                if file.is_generated {
                    continue;
                }
                let mut visitor = ConsumptionVisitor {
                    file,
                    allowed: false,
                    spans: Vec::new(),
                    composed_steps: Vec::new(),
                };
                visitor.visit_file(&visitor.file.syntax);
                let composed_steps = visitor.list_outermost_steps();
                for span in visitor.spans {
                    findings.push(build_node_finding("ResultConsumption", &file.path, find_node_span(span),
                        format_module_path(&file.module.crate_name, &file.module.path),
                        "Scope let_underscore_must_use expectations to the intentional discard statement (error-handling.rs.md)".to_owned()));
                }
                for span in composed_steps {
                    findings.push(build_node_finding(
                        "ResultConsumption",
                        &file.path,
                        span,
                        format_module_path(&file.module.crate_name, &file.module.path),
                        "Name each fallible step in its own let before composing the next step (error-handling.rs.md)".to_owned(),
                    ));
                }
            }
        }
        findings.extend(TypedRuntimeVisitor::check_context(
            context,
            "ResultConsumption",
        ));
        findings
    }
}
