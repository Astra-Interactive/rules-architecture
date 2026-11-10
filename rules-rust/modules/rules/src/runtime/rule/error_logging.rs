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
    results: Vec<ResultBinding>,
    propagating: bool,
}

impl LogVisitor<'_> {
    fn result_type(&self, ordinal: usize, module: usize, typed: &syn::Type) -> bool {
        self.context
            .resolve_outer_external_type_paths(ordinal, module, typed)
            .iter()
            .any(|path| {
                matches!(
                    path.format_path().as_str(),
                    "std::result::Result" | "core::result::Result"
                )
            })
    }

    fn mask_result_pattern(&mut self, pattern: &syn::Pat) {
        match pattern {
            syn::Pat::Ident(name) => self.results.push(ResultBinding {
                name: name.ident.to_string(),
                is_result: false,
                typed: None,
            }),
            syn::Pat::Type(pattern) => {
                self.mask_result_pattern(&pattern.pat);
                let result = self.result_type(self.ordinal, self.module, &pattern.ty);
                if let Some(binding) = self.results.last_mut() {
                    binding.is_result = result;
                    binding.typed = Some((*pattern.ty).clone());
                }
            }
            syn::Pat::Paren(pattern) => self.mask_result_pattern(&pattern.pat),
            syn::Pat::Reference(pattern) => self.mask_result_pattern(&pattern.pat),
            syn::Pat::Tuple(pattern) => {
                for pattern in &pattern.elems {
                    self.mask_result_pattern(pattern);
                }
            }
            syn::Pat::TupleStruct(pattern) => {
                for pattern in &pattern.elems {
                    self.mask_result_pattern(pattern);
                }
            }
            syn::Pat::Struct(pattern) => {
                for field in &pattern.fields {
                    self.mask_result_pattern(&field.pat);
                }
            }
            syn::Pat::Slice(pattern) => {
                for pattern in &pattern.elems {
                    self.mask_result_pattern(pattern);
                }
            }
            syn::Pat::Or(pattern) => {
                for pattern in &pattern.cases {
                    self.mask_result_pattern(pattern);
                }
            }
            _ => {}
        }
    }

    fn resolve_expression_type(&self, expression: &syn::Expr) -> Option<syn::Type> {
        match expression {
            syn::Expr::Path(path) => self
                .results
                .iter()
                .rev()
                .find(|binding| path.path.is_ident(&binding.name))
                .and_then(|binding| binding.typed.clone()),
            syn::Expr::Paren(paren) => self.resolve_expression_type(&paren.expr),
            syn::Expr::Group(group) => self.resolve_expression_type(&group.expr),
            _ => None,
        }
    }

    fn is_string_receiver(&self, expression: &syn::Expr) -> bool {
        let Some(typed) = self.resolve_expression_type(expression) else {
            return false;
        };
        let cfg = crate::runtime::util::node_predicate(
            &self.context.indexes[self.ordinal],
            self.module,
            &self.file.path,
            find_node_span(expression.span()),
        );
        let resolved =
            self.context
                .resolve_type_syntax_under(self.ordinal, self.module, &typed, &cfg);
        !resolved.is_empty() && resolved.iter().all(|resolved| {
            if resolved.has_unsubstituted_parameters { return false; }
            let mut typed = &resolved.typed;
            while let syn::Type::Reference(reference) = typed { typed = &reference.elem; }
            matches!(typed, syn::Type::Path(path) if path.path.is_ident("str")
                && matches!(self.context.indexes[resolved.index].resolve_path(resolved.module, &path.path), crate::source::model::Resolution::Local))
                || self.context.resolve_outer_external_type_paths_under(resolved.index, resolved.module, typed, &resolved.cfg)
                    .iter().any(|path| matches!(path.format_path().as_str(), "std::primitive::str" | "core::primitive::str"))
        })
    }

    fn is_result_call(&self, call: &syn::ExprCall) -> bool {
        let syn::Expr::Path(path) = &*call.func else {
            return false;
        };
        if self
            .results
            .iter()
            .rev()
            .any(|binding| path.path.is_ident(&binding.name))
        {
            return false;
        }
        if ["Ok", "Err"].iter().any(|variant| {
            crate::runtime::util::is_result_variant(
                self.context,
                self.ordinal,
                self.module,
                &path.path,
                variant,
            )
        }) {
            return true;
        }
        self.context.resolve_workspace_path(self.ordinal, self.module, &path.path).iter().any(|target| {
            let Some(identity) = target.path.item else { return false; };
            let item = &self.context.indexes[target.index].items[identity];
            target.path.cfg.and(&item.cfg).is_satisfiable() && matches!(&item.syntax, syn::Item::Fn(function) if matches!(&function.sig.output, syn::ReturnType::Type(_, typed) if self.result_type(target.index, item.module, typed)))
        }) || self.context.resolve_external_call_paths(self.ordinal, self.module, &path.path).iter().any(|path| {
            matches!(path.format_path().as_str(), "std::fs::read" | "std::fs::read_to_string" | "std::fs::write" | "tokio::fs::read" | "tokio::fs::read_to_string" | "tokio::fs::write")
        })
    }

    fn is_standard_call(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Paren(paren) => self.is_standard_call(&paren.expr),
            syn::Expr::Group(group) => self.is_standard_call(&group.expr),
            syn::Expr::Call(call) => {
                let syn::Expr::Path(path) = &*call.func else {
                    return false;
                };
                self.context
                    .resolve_external_call_paths(self.ordinal, self.module, &path.path)
                    .iter()
                    .any(|path| matches!(path.package.as_str(), "std" | "core" | "alloc"))
            }
            _ => false,
        }
    }

    fn result_expression(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Paren(paren) => self.result_expression(&paren.expr),
            syn::Expr::Group(group) => self.result_expression(&group.expr),
            syn::Expr::Await(awaited) => self.result_expression(&awaited.base),
            syn::Expr::Reference(reference) => self.result_expression(&reference.expr),
            syn::Expr::Path(path) => self
                .results
                .iter()
                .rev()
                .find(|binding| path.path.is_ident(&binding.name))
                .is_some_and(|binding| binding.is_result),
            syn::Expr::MethodCall(call) => {
                if call.method == "parse" && self.is_string_receiver(&call.receiver) {
                    return true;
                }
                [
                    "map",
                    "map_err",
                    "inspect",
                    "inspect_err",
                    "and_then",
                    "or_else",
                    "as_ref",
                    "as_mut",
                ]
                .iter()
                .any(|name| call.method == *name)
                    && self.result_expression(&call.receiver)
            }
            syn::Expr::Call(call) => self.is_result_call(call),
            _ => false,
        }
    }

    fn bind_results(&mut self, signature: &syn::Signature) {
        self.results.clear();
        for argument in &signature.inputs {
            if let syn::FnArg::Typed(parameter) = argument
                && let syn::Pat::Ident(name) = &*parameter.pat
            {
                self.results.push(ResultBinding {
                    name: name.ident.to_string(),
                    is_result: self.result_type(self.ordinal, self.module, &parameter.ty),
                    typed: Some((*parameter.ty).clone()),
                });
            }
        }
    }

    fn error_name(&self, pattern: &syn::Pat) -> Option<String> {
        match pattern {
            syn::Pat::Paren(pattern) => self.error_name(&pattern.pat),
            syn::Pat::Reference(pattern) => self.error_name(&pattern.pat),
            syn::Pat::TupleStruct(pattern) => {
                let error = crate::runtime::util::is_result_variant(
                    self.context,
                    self.ordinal,
                    self.module,
                    &pattern.path,
                    "Err",
                );
                if error && let Some(syn::Pat::Ident(name)) = pattern.elems.first() {
                    return Some(name.ident.to_string());
                }
                pattern
                    .elems
                    .iter()
                    .find_map(|element| self.error_name(element))
            }
            _ => None,
        }
    }

    fn bind_error(&mut self, pattern: &syn::Pat) {
        if let Some(name) = self.error_name(pattern) {
            self.logged.retain(|(logged, _)| logged != &name);
            if !self.errors.contains(&name) {
                self.errors.push(name);
            }
        }
    }

    fn bind_condition(&mut self, condition: &syn::Expr) -> Vec<String> {
        match condition {
            syn::Expr::Let(condition) => {
                self.mask_result_pattern(&condition.pat);
                self.bind_error(&condition.pat);
                let mut bindings = Vec::new();
                bindings.extend(self.error_name(&condition.pat));
                if let syn::Pat::TupleStruct(pattern) = &*condition.pat
                    && let Some(syn::Pat::Ident(name)) = pattern.elems.first()
                {
                    bindings.push(name.ident.to_string());
                }
                bindings
            }
            syn::Expr::Binary(binary) if matches!(binary.op, syn::BinOp::And(_)) => {
                let mut bindings = self.bind_condition(&binary.left);
                bindings.extend(self.bind_condition(&binary.right));
                bindings
            }
            syn::Expr::Paren(paren) => self.bind_condition(&paren.expr),
            _ => Vec::new(),
        }
    }

    fn scoped_function(&mut self, signature: &syn::Signature, visit: impl FnOnce(&mut Self)) {
        let errors = std::mem::take(&mut self.errors);
        let logged = std::mem::take(&mut self.logged);
        let results = std::mem::take(&mut self.results);
        self.bind_results(signature);
        visit(self);
        self.errors = errors;
        self.logged = logged;
        self.results = results;
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
            let mut expression = &expression;
            loop {
                expression = match expression {
                    syn::Expr::Paren(paren) => &paren.expr,
                    syn::Expr::Group(group) => &group.expr,
                    _ => break,
                };
            }
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
        let result = match &local.pat {
            syn::Pat::Type(typed) => self.result_type(self.ordinal, self.module, &typed.ty),
            _ => local
                .init
                .as_ref()
                .is_some_and(|init| self.result_expression(&init.expr)),
        };
        let pattern = match &local.pat {
            syn::Pat::Type(typed) => &*typed.pat,
            pattern => pattern,
        };
        let typed = match &local.pat {
            syn::Pat::Type(pattern) => Some((*pattern.ty).clone()),
            _ => local
                .init
                .as_ref()
                .and_then(|init| self.resolve_expression_type(&init.expr)),
        };
        self.mask_result_pattern(&local.pat);
        if local
            .init
            .as_ref()
            .is_some_and(|init| init.diverge.is_some())
        {
            self.bind_error(pattern);
        }
        if let syn::Pat::Ident(name) = pattern {
            self.results.push(ResultBinding {
                name: name.ident.to_string(),
                is_result: result,
                typed,
            });
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
        self.scoped_function(&item.sig, |visitor| {
            syn::visit::visit_item_fn(visitor, item);
        });
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        self.scoped_function(&item.sig, |visitor| {
            syn::visit::visit_impl_item_fn(visitor, item);
        });
    }

    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        let results = self.results.len();
        self.mask_result_pattern(&arm.pat);
        let length = self.errors.len();
        let logged = self.logged.clone();
        self.bind_error(&arm.pat);
        syn::visit::visit_arm(self, arm);
        self.errors.truncate(length);
        self.logged = logged;
        self.results.truncate(results);
    }

    fn visit_expr_closure(&mut self, expression: &'ast syn::ExprClosure) {
        let results = self.results.len();
        for parameter in &expression.inputs {
            self.mask_result_pattern(parameter);
        }
        syn::visit::visit_expr_closure(self, expression);
        self.results.truncate(results);
    }

    fn visit_expr_for_loop(&mut self, expression: &'ast syn::ExprForLoop) {
        self.visit_expr(&expression.expr);
        let results = self.results.len();
        self.mask_result_pattern(&expression.pat);
        self.visit_block(&expression.body);
        self.results.truncate(results);
    }

    fn visit_expr_if(&mut self, expression: &'ast syn::ExprIf) {
        let results = self.results.len();
        let length = self.errors.len();
        let logged = self.logged.clone();
        self.visit_expr(&expression.cond);
        let bindings = self.bind_condition(&expression.cond);
        self.visit_block(&expression.then_branch);
        let then_logged = self.logged.clone();
        self.errors.truncate(length);
        self.logged = logged;
        self.results.truncate(results);
        if let Some((_, otherwise)) = &expression.else_branch {
            self.visit_expr(otherwise);
        }
        self.logged.extend(
            then_logged
                .into_iter()
                .filter(|(name, _)| self.errors.contains(name) && !bindings.contains(name)),
        );
    }

    fn visit_expr_while(&mut self, expression: &'ast syn::ExprWhile) {
        self.visit_expr(&expression.cond);
        let results = self.results.len();
        let length = self.errors.len();
        let logged = self.logged.clone();
        self.bind_condition(&expression.cond);
        self.visit_block(&expression.body);
        self.errors.truncate(length);
        self.logged = logged;
        self.results.truncate(results);
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
        let result = self.result_expression(&call.receiver)
            || call.method == "map_err" && self.is_standard_call(&call.receiver);
        if result
            && matches!(
                call.method.to_string().as_str(),
                "map_err" | "inspect_err" | "unwrap_or_else" | "or_else" | "map_or_else"
            )
        {
            self.visit_expr(&call.receiver);
            for (position, argument) in call.args.iter().enumerate() {
                let errors = self.errors.clone();
                let logged = self.logged.clone();
                if position == 0
                    && let syn::Expr::Closure(closure) = argument
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
        let results = self.results.len();
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
        self.results.truncate(results);
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

struct ResultBinding {
    name: String,
    is_result: bool,
    typed: Option<syn::Type>,
}
