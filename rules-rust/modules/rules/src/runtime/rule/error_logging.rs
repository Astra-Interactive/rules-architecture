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
    results: Vec<String>,
    propagating: bool,
}

impl LogVisitor<'_> {
    fn bind_results(&mut self, signature: &syn::Signature) {
        self.results.clear();
        for argument in &signature.inputs {
            if let syn::FnArg::Typed(parameter) = argument
                && let syn::Pat::Ident(name) = &*parameter.pat
                && self
                    .context
                    .resolve_outer_external_type_paths(self.ordinal, self.module, &parameter.ty)
                    .iter()
                    .any(|path| {
                        matches!(
                            path.format_path().as_str(),
                            "std::result::Result" | "core::result::Result"
                        )
                    })
            {
                self.results.push(name.ident.to_string());
            }
        }
    }
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
            let name = name.ident.to_string();
            self.logged.retain(|(logged, _)| logged != &name);
            if !self.errors.contains(&name) {
                self.errors.push(name);
            }
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

    fn field_preserves_error(&self, field: &[proc_macro2::TokenTree], name: &str) -> bool {
        let Some(assignment) = field.iter().position(
            |token| matches!(token, proc_macro2::TokenTree::Punct(punct) if punct.as_char() == '='),
        ) else {
            return false;
        };
        let value = field[assignment + 1..]
            .iter()
            .cloned()
            .collect::<proc_macro2::TokenStream>();
        if !Self::token_names(value.clone())
            .iter()
            .any(|value| value == name)
        {
            return false;
        }
        let error_cast = syn::parse2::<syn::Expr>(value).is_ok_and(|expression| {
            let syn::Expr::Cast(cast) = expression else {
                return false;
            };
            self.context
                .resolve_external_type_paths(self.ordinal, self.module, &cast.ty)
                .iter()
                .any(|path| {
                    matches!(
                        path.format_path().as_str(),
                        "std::error::Error" | "core::error::Error"
                    )
                })
        });
        let error_capture = field[..assignment].windows(2).any(|pair| {
            matches!(&pair[0], proc_macro2::TokenTree::Punct(punct) if punct.as_char() == ':')
                && matches!(&pair[1], proc_macro2::TokenTree::Ident(ident) if ident == "err")
        });
        error_cast || error_capture
    }

    fn error_field(&self, tokens: proc_macro2::TokenStream, name: &str) -> bool {
        let mut field = Vec::new();
        for token in tokens
            .into_iter()
            .chain(std::iter::once(proc_macro2::TokenTree::Punct(
                proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone),
            )))
        {
            if matches!(&token, proc_macro2::TokenTree::Punct(punct) if matches!(punct.as_char(), ',' | ';'))
            {
                if self.field_preserves_error(&field, name) {
                    return true;
                }
                field.clear();
            } else {
                field.push(token);
            }
        }
        false
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
        let results = std::mem::take(&mut self.results);
        self.bind_results(&item.sig);
        syn::visit::visit_item_fn(self, item);
        self.errors = errors;
        self.logged = logged;
        self.results = results;
    }

    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        let length = self.errors.len();
        let logged = self.logged.clone();
        self.bind_error(&arm.pat);
        syn::visit::visit_arm(self, arm);
        self.errors.truncate(length);
        self.logged = logged;
    }

    fn visit_expr_if(&mut self, expression: &'ast syn::ExprIf) {
        let length = self.errors.len();
        let logged = self.logged.clone();
        self.visit_expr(&expression.cond);
        let mut binding = None;
        if let syn::Expr::Let(condition) = &*expression.cond {
            self.bind_error(&condition.pat);
            if let syn::Pat::TupleStruct(pattern) = &*condition.pat
                && let Some(syn::Pat::Ident(name)) = pattern.elems.first()
            {
                binding = Some(name.ident.to_string());
            }
        }
        self.visit_block(&expression.then_branch);
        let then_logged = self.logged.clone();
        self.errors.truncate(length);
        self.logged = logged;
        if let Some((_, otherwise)) = &expression.else_branch {
            self.visit_expr(otherwise);
        }
        self.logged.extend(
            then_logged
                .into_iter()
                .filter(|(name, _)| self.errors.contains(name) && binding.as_ref() != Some(name)),
        );
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
            let mentions = outside || Self::interpolation(item.tokens.clone(), name);
            if mentions && !self.error_field(item.tokens.clone(), name) {
                self.spans.push(item.span());
            }
            if mentions {
                if self.propagating {
                    self.spans.push(item.span());
                }
                if self.logged.iter().any(|(logged, _)| logged == name) {
                    self.spans.push(item.span());
                }
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
        let parent = self.propagating;
        self.propagating = true;
        syn::visit::visit_expr_try(self, expression);
        self.propagating = parent;
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        let result = matches!(&*call.receiver, syn::Expr::Path(path) if self.results.iter().any(|name| path.path.is_ident(name)));
        if result && matches!(call.method.to_string().as_str(), "map_err" | "inspect_err") {
            self.visit_expr(&call.receiver);
            for argument in &call.args {
                let errors = self.errors.clone();
                let logged = self.logged.clone();
                if let syn::Expr::Closure(closure) = argument
                    && let Some(syn::Pat::Ident(name)) = closure.inputs.first()
                {
                    self.errors.push(name.ident.to_string());
                }
                self.visit_expr(argument);
                self.errors = errors;
                self.logged = logged;
            }
            return;
        }
        syn::visit::visit_expr_method_call(self, call);
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        let logged = self.logged.clone();
        let errors = self.errors.clone();
        syn::visit::visit_block(self, block);
        if let Some(syn::Stmt::Expr(expression, None)) = block.stmts.last() {
            self.propagate(expression);
        }
        let retained = self
            .logged
            .iter()
            .filter(|(name, _)| errors.contains(name))
            .cloned()
            .collect::<Vec<_>>();
        self.logged = logged;
        self.logged.extend(retained);
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
                    results: Vec::new(),
                    propagating: false,
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
