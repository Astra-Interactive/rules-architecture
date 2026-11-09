use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::{build_node_finding, format_module_path};
use crate::source::model::SourceFile;
use crate::source::util::find_node_span;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct ErrorLogging;

struct LogVisitor<'source> {
    context: &'source RuleContext,
    ordinal: usize,
    module: usize,
    file: &'source SourceFile,
    errors: Vec<String>,
    logged: Vec<(String, proc_macro2::Span)>,
    spans: Vec<proc_macro2::Span>,
}

impl LogVisitor<'_> {
    fn bind_error(&mut self, pattern: &syn::Pat) {
        if let syn::Pat::TupleStruct(pattern) = pattern
            && crate::runtime::util::is_result_variant(
                self.context,
                self.ordinal,
                self.module,
                &pattern.path,
                "Err",
            )
            && let Some(syn::Pat::Ident(name)) = pattern.elems.first()
        {
            self.errors.push(name.ident.to_string());
        }
    }

    fn token_names(tokens: proc_macro2::TokenStream) -> Vec<String> {
        tokens
            .into_iter()
            .flat_map(|token| match token {
                proc_macro2::TokenTree::Ident(name) => vec![name.to_string()],
                proc_macro2::TokenTree::Group(group) => Self::token_names(group.stream()),
                _ => Vec::new(),
            })
            .collect()
    }

    fn interpolation(tokens: proc_macro2::TokenStream, name: &str) -> bool {
        tokens.into_iter().any(|token| match token {
            proc_macro2::TokenTree::Literal(literal) => {
                syn::parse_str::<syn::LitStr>(&literal.to_string()).is_ok_and(|value| {
                    value.value().contains(&format!("{{{name}}}"))
                        || value.value().contains(&format!("{{{name}:"))
                })
            }
            proc_macro2::TokenTree::Group(group) => Self::interpolation(group.stream(), name),
            _ => false,
        })
    }

    fn propagate(&mut self, expression: &syn::Expr) {
        let syn::Expr::Call(call) = expression else {
            return;
        };
        if !matches!(&*call.func, syn::Expr::Path(path) if crate::runtime::util::is_result_variant(self.context, self.ordinal, self.module, &path.path, "Err"))
        {
            return;
        }
        for argument in &call.args {
            let syn::Expr::Path(path) = argument else {
                continue;
            };
            self.spans.extend(
                self.logged
                    .iter()
                    .filter(|(name, _)| path.path.is_ident(name))
                    .map(|(_, span)| *span),
            );
        }
    }
}

impl<'ast> Visit<'ast> for LogVisitor<'_> {
    fn visit_local(&mut self, local: &'ast syn::Local) {
        syn::visit::visit_local(self, local);
        let pattern = match &local.pat {
            syn::Pat::Type(typed) => &*typed.pat,
            pattern => pattern,
        };
        if let syn::Pat::Ident(name) = pattern {
            self.errors.retain(|error| name.ident != error);
            self.logged.retain(|(error, _)| name.ident != error);
        }
    }
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        let parent = self.module;
        if let Some(module) = self.context.indexes[self.ordinal]
            .modules
            .iter()
            .find(|module| {
                module.parent == Some(parent) && module.span == find_node_span(item.span())
            })
        {
            self.module = module.id;
        }
        syn::visit::visit_item_mod(self, item);
        self.module = parent;
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        let errors = std::mem::take(&mut self.errors);
        let logged = std::mem::take(&mut self.logged);
        syn::visit::visit_item_fn(self, item);
        self.errors = errors;
        self.logged = logged;
    }

    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        let length = self.errors.len();
        self.bind_error(&arm.pat);
        syn::visit::visit_arm(self, arm);
        self.errors.truncate(length);
    }

    fn visit_expr_if(&mut self, expression: &'ast syn::ExprIf) {
        let length = self.errors.len();
        self.visit_expr(&expression.cond);
        if let syn::Expr::Let(condition) = &*expression.cond {
            self.bind_error(&condition.pat);
        }
        self.visit_block(&expression.then_branch);
        self.errors.truncate(length);
        if let Some((_, otherwise)) = &expression.else_branch {
            self.visit_expr(otherwise);
        }
    }

    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        let logging = self
            .context
            .resolve_external_call_paths(self.ordinal, self.module, &item.path)
            .iter()
            .any(|path| matches!(path.package.as_str(), "tracing" | "log"));
        if !logging {
            return;
        }
        let names = Self::token_names(item.tokens.clone());
        for name in &self.errors {
            let outside = names.contains(name);
            if Self::interpolation(item.tokens.clone(), name) && !outside {
                self.spans.push(item.span());
            }
            if outside {
                self.logged.push((name.clone(), item.span()));
            }
        }
    }

    fn visit_expr_return(&mut self, expression: &'ast syn::ExprReturn) {
        if let Some(value) = &expression.expr {
            self.propagate(value);
        }
        syn::visit::visit_expr_return(self, expression);
    }

    fn visit_expr_try(&mut self, expression: &'ast syn::ExprTry) {
        self.propagate(&expression.expr);
        if let syn::Expr::Path(path) = &*expression.expr {
            self.spans.extend(
                self.logged
                    .iter()
                    .filter(|(name, _)| path.path.is_ident(name))
                    .map(|(_, span)| *span),
            );
        }
        syn::visit::visit_expr_try(self, expression);
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        let logged = self.logged.clone();
        let errors = self.errors.clone();
        syn::visit::visit_block(self, block);
        if let Some(syn::Stmt::Expr(expression, None)) = block.stmts.last() {
            self.propagate(expression);
        }
        self.logged = logged;
        self.errors = errors;
    }
}

impl Rule for ErrorLogging {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for sources in &context.crates {
            for file in &sources.files {
                if file.is_generated {
                    continue;
                }
                let Some((ordinal, index)) =
                    context.indexes.iter().enumerate().find(|(_, index)| {
                        index.crate_name == file.module.crate_name
                            && index.target == file.module.target
                    })
                else {
                    continue;
                };
                let Some(module) = index
                    .modules
                    .iter()
                    .find(|module| module.module == file.module)
                else {
                    continue;
                };
                let mut visitor = LogVisitor {
                    context,
                    ordinal,
                    module: module.id,
                    file,
                    errors: Vec::new(),
                    logged: Vec::new(),
                    spans: Vec::new(),
                };
                visitor.visit_file(&visitor.file.syntax);
                visitor
                    .spans
                    .sort_by_key(|span| (span.start().line, span.start().column));
                visitor
                    .spans
                    .dedup_by_key(|span| (span.start().line, span.start().column));
                for span in visitor.spans.into_iter().filter(|span| {
                    crate::runtime::util::node_predicate(
                        index,
                        module.id,
                        &file.path,
                        find_node_span(*span),
                    )
                    .is_satisfiable()
                }) {
                    findings.push(build_node_finding("ErrorLogging", &file.path, find_node_span(span),
                format_module_path(&index.crate_name, &file.module.path), "Pass the error as an event field and consume it once (no-print.rs.md; error-handling.rs.md)".to_owned()));
                }
            }
        }
        findings
    }
}
