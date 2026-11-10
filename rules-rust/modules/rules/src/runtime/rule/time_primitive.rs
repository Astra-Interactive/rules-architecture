use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::{build_node_finding, format_module_path};
use crate::runtime::util::node_predicate;
use crate::source::util::find_node_span;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct TimePrimitive;

struct TimeVisitor<'context> {
    context: &'context RuleContext,
    ordinal: usize,
    module: usize,
    spans: Vec<proc_macro2::Span>,
    bindings: Vec<(String, bool, bool)>,
}

impl TimeVisitor<'_> {
    fn bind_signature(&mut self, signature: &syn::Signature) {
        self.bindings.clear();
        for parameter in &signature.inputs {
            let syn::FnArg::Typed(parameter) = parameter else {
                continue;
            };
            self.bind_pattern(&parameter.pat, Some(&parameter.ty), false);
        }
    }

    fn sender_type(&self, typed: &syn::Type) -> bool {
        let typed = match typed {
            syn::Type::Reference(reference) => &*reference.elem,
            typed => typed,
        };
        self.context
            .resolve_outer_external_type_paths(self.ordinal, self.module, typed)
            .iter()
            .any(|path| {
                [
                    "tokio::sync::mpsc::Sender",
                    "tokio::sync::mpsc::UnboundedSender",
                    "tokio::sync::oneshot::Sender",
                    "tokio::sync::broadcast::Sender",
                    "tokio::sync::watch::Sender",
                    "std::sync::mpsc::Sender",
                    "std::sync::mpsc::SyncSender",
                ]
                .contains(&path.format_path().as_str())
            })
    }

    fn bind_pattern(&mut self, pattern: &syn::Pat, typed: Option<&syn::Type>, instant: bool) {
        match pattern {
            syn::Pat::Ident(name) => {
                self.bindings.push((
                    name.ident.to_string(),
                    instant || typed.is_some_and(|typed| self.instant(typed)),
                    typed.is_some_and(|typed| self.sender_type(typed)),
                ));
                if let Some((_, pattern)) = &name.subpat {
                    self.bind_pattern(pattern, None, false);
                }
            }
            syn::Pat::Type(pattern) => self.bind_pattern(&pattern.pat, Some(&pattern.ty), instant),
            syn::Pat::Paren(pattern) => self.bind_pattern(&pattern.pat, typed, instant),
            syn::Pat::Reference(pattern) => self.bind_pattern(&pattern.pat, None, false),
            syn::Pat::Tuple(pattern) => {
                for pattern in &pattern.elems {
                    self.bind_pattern(pattern, None, false);
                }
            }
            syn::Pat::TupleStruct(pattern) => {
                for pattern in &pattern.elems {
                    self.bind_pattern(pattern, None, false);
                }
            }
            syn::Pat::Slice(pattern) => {
                for pattern in &pattern.elems {
                    self.bind_pattern(pattern, None, false);
                }
            }
            syn::Pat::Struct(pattern) => {
                for field in &pattern.fields {
                    self.bind_pattern(&field.pat, None, false);
                }
            }
            syn::Pat::Or(pattern) => {
                for pattern in &pattern.cases {
                    self.bind_pattern(pattern, None, false);
                }
            }
            _ => syn::visit::visit_pat(self, pattern),
        }
    }

    fn visit_condition(&mut self, condition: &syn::Expr) {
        match condition {
            syn::Expr::Let(condition) => {
                self.visit_expr(&condition.expr);
                let instant = self.local_instant(&condition.expr);
                self.bind_pattern(&condition.pat, None, instant);
            }
            syn::Expr::Binary(binary) if matches!(binary.op, syn::BinOp::And(_)) => {
                self.visit_condition(&binary.left);
                self.visit_condition(&binary.right);
            }
            _ => self.visit_expr(condition),
        }
    }

    fn instant(&self, typed: &syn::Type) -> bool {
        let index = &self.context.indexes[self.ordinal];
        let cfg = node_predicate(
            index,
            self.module,
            &index.modules[self.module].file,
            find_node_span(typed.span()),
        );
        self.context
            .resolve_external_type_paths_under(self.ordinal, self.module, typed, &cfg)
            .iter()
            .any(|path| {
                matches!(
                    path.format_path().as_str(),
                    "std::time::Instant" | "tokio::time::Instant"
                )
            })
    }

    fn primitive(&self, typed: &syn::Type) -> bool {
        if self
            .context
            .resolve_external_type_paths(self.ordinal, self.module, typed)
            .iter()
            .any(|path| {
                matches!(
                    path.format_path().as_str(),
                    "std::string::String" | "alloc::string::String"
                )
            })
        {
            return true;
        }
        if let syn::Type::Reference(reference) = typed {
            return self.primitive(&reference.elem);
        }
        self.context.resolve_type_syntax(self.ordinal, self.module, typed).iter().any(|resolved|
            matches!(&resolved.typed, syn::Type::Path(path) if path.path.get_ident().is_some_and(|name|
                ["u8","u16","u32","u64","u128","usize","i8","i16","i32","i64","i128","isize","f32","f64","str"].contains(&name.to_string().as_str()))))
    }

    fn unit_constant(&self, ident: &syn::Ident, typed: &syn::Type) -> bool {
        [
            "_MS", "_MILLIS", "_SECS", "_SECONDS", "_NANOS", "_MICROS", "_MINUTES", "_HOURS",
        ]
        .iter()
        .any(|suffix| ident.to_string().ends_with(suffix))
            && self.primitive(typed)
    }

    fn local_instant(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Path(path) => self
                .bindings
                .iter()
                .rev()
                .find(|(name, _, _)| path.path.is_ident(name))
                .is_some_and(|(_, instant, _)| *instant),
            syn::Expr::Reference(reference) => self.local_instant(&reference.expr),
            syn::Expr::Paren(paren) => self.local_instant(&paren.expr),
            syn::Expr::Cast(cast) => self.local_instant(&cast.expr),
            syn::Expr::Call(call) => {
                matches!(&*call.func, syn::Expr::Path(path) if self.context.resolve_external_call_paths(self.ordinal, self.module, &path.path).iter().any(|path| matches!(path.format_path().as_str(), "std::time::Instant::now" | "tokio::time::Instant::now")))
            }
            _ => false,
        }
    }

    fn logged_instant(&self, field: &[proc_macro2::TokenTree]) -> bool {
        let assignment = field.iter().position(
            |token| matches!(token, proc_macro2::TokenTree::Punct(punct) if punct.as_char() == '='),
        );
        let mut value = &field[assignment.map_or(0, |position| position + 1)..];
        if matches!(value.first(), Some(proc_macro2::TokenTree::Punct(punct)) if matches!(punct.as_char(), '?' | '%'))
        {
            value = &value[1..];
        }
        let tokens = value.iter().cloned().collect::<proc_macro2::TokenStream>();
        let Ok(expression) = syn::parse2::<syn::Expr>(tokens) else {
            return false;
        };
        if self.local_instant(&expression) {
            return true;
        }
        let syn::Expr::Lit(literal) = expression else {
            return false;
        };
        let syn::Lit::Str(message) = literal.lit else {
            return false;
        };
        self.bindings.iter().any(|(name, _, _)| {
            self.bindings
                .iter()
                .rev()
                .find(|(binding, _, _)| binding == name)
                .is_some_and(|(_, instant, _)| *instant)
                && (message.value().contains(&format!("{{{name}}}"))
                    || message.value().contains(&format!("{{{name}:")))
        })
    }

    fn logs_instant(&self, tokens: proc_macro2::TokenStream) -> bool {
        let mut field = Vec::new();
        for token in tokens
            .into_iter()
            .chain(std::iter::once(proc_macro2::TokenTree::Punct(
                proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone),
            )))
        {
            if matches!(&token, proc_macro2::TokenTree::Punct(punct) if matches!(punct.as_char(), ',' | ';'))
            {
                if self.logged_instant(&field) {
                    return true;
                }
                field.clear();
            } else {
                field.push(token);
            }
        }
        false
    }
}

impl<'ast> Visit<'ast> for TimeVisitor<'_> {
    fn visit_pat_ident(&mut self, pattern: &'ast syn::PatIdent) {
        self.bind_pattern(&syn::Pat::Ident(pattern.clone()), None, false);
    }

    fn visit_signature(&mut self, signature: &'ast syn::Signature) {
        self.visit_return_type(&signature.output);
    }
    fn visit_expr_closure(&mut self, expression: &'ast syn::ExprClosure) {
        let length = self.bindings.len();
        for parameter in &expression.inputs {
            self.bind_pattern(parameter, None, false);
        }
        self.visit_expr(&expression.body);
        self.bindings.truncate(length);
    }

    fn visit_expr_for_loop(&mut self, expression: &'ast syn::ExprForLoop) {
        self.visit_expr(&expression.expr);
        let length = self.bindings.len();
        self.bind_pattern(&expression.pat, None, false);
        self.visit_block(&expression.body);
        self.bindings.truncate(length);
    }

    fn visit_expr_if(&mut self, expression: &'ast syn::ExprIf) {
        let length = self.bindings.len();
        self.visit_condition(&expression.cond);
        self.visit_block(&expression.then_branch);
        self.bindings.truncate(length);
        if let Some((_, otherwise)) = &expression.else_branch {
            self.visit_expr(otherwise);
        }
    }

    fn visit_expr_while(&mut self, expression: &'ast syn::ExprWhile) {
        let length = self.bindings.len();
        self.visit_condition(&expression.cond);
        self.visit_block(&expression.body);
        self.bindings.truncate(length);
    }

    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        let length = self.bindings.len();
        self.bind_pattern(&arm.pat, None, false);
        self.visit_expr(&arm.body);
        self.bindings.truncate(length);
    }
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        let bindings = std::mem::take(&mut self.bindings);
        self.bind_signature(&item.sig);
        syn::visit::visit_item_fn(self, item);
        self.bindings = bindings;
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        let bindings = std::mem::take(&mut self.bindings);
        self.bind_signature(&item.sig);
        syn::visit::visit_impl_item_fn(self, item);
        self.bindings = bindings;
    }
    fn visit_block(&mut self, block: &'ast syn::Block) {
        let length = self.bindings.len();
        syn::visit::visit_block(self, block);
        self.bindings.truncate(length);
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let Some(init) = &local.init {
            self.visit_expr(&init.expr);
            if let Some((_, diverge)) = &init.diverge {
                self.visit_expr(diverge);
            }
        }
        let instant = local
            .init
            .as_ref()
            .is_some_and(|init| self.local_instant(&init.expr));
        self.bind_pattern(&local.pat, None, instant);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        let elapsed_operation = matches!(
            call.method.to_string().as_str(),
            "duration_since" | "checked_duration_since" | "saturating_duration_since"
        ) && self.local_instant(&call.receiver);
        let send = matches!(
            call.method.to_string().as_str(),
            "send" | "try_send" | "send_replace"
        ) && matches!(&*call.receiver, syn::Expr::Path(path) if self.bindings.iter().rev().find(|(name, _, _)| path.path.is_ident(name)).is_some_and(|(_, _, sender)| *sender));
        if !elapsed_operation
            && send
            && call
                .args
                .iter()
                .any(|argument| self.local_instant(argument))
        {
            self.spans.push(call.span());
        }
        syn::visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_struct(&mut self, expression: &'ast syn::ExprStruct) {
        for field in &expression.fields {
            if self.local_instant(&field.expr) {
                self.spans.push(field.span());
            }
        }
        syn::visit::visit_expr_struct(self, expression);
    }

    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        let logging = self
            .context
            .resolve_external_call_paths(self.ordinal, self.module, &item.path)
            .iter()
            .any(|path| matches!(path.package.as_str(), "tracing" | "log"));
        if logging && self.logs_instant(item.tokens.clone()) {
            self.spans.push(item.span());
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

    fn visit_field(&mut self, field: &'ast syn::Field) {
        if self.instant(&field.ty) {
            self.spans.push(field.span());
        }
        syn::visit::visit_field(self, field);
    }

    fn visit_return_type(&mut self, returned: &'ast syn::ReturnType) {
        if let syn::ReturnType::Type(_, typed) = returned
            && self.instant(typed)
        {
            self.spans.push(returned.span());
        }
        syn::visit::visit_return_type(self, returned);
    }

    fn visit_item_const(&mut self, item: &'ast syn::ItemConst) {
        if self.unit_constant(&item.ident, &item.ty) {
            self.spans.push(item.span());
        }
        syn::visit::visit_item_const(self, item);
    }

    fn visit_item_static(&mut self, item: &'ast syn::ItemStatic) {
        if self.unit_constant(&item.ident, &item.ty) {
            self.spans.push(item.span());
        }
        syn::visit::visit_item_static(self, item);
    }

    fn visit_impl_item_const(&mut self, item: &'ast syn::ImplItemConst) {
        if self.unit_constant(&item.ident, &item.ty) {
            self.spans.push(item.span());
        }
        syn::visit::visit_impl_item_const(self, item);
    }

    fn visit_trait_item_const(&mut self, item: &'ast syn::TraitItemConst) {
        if self.unit_constant(&item.ident, &item.ty) {
            self.spans.push(item.span());
        }
        syn::visit::visit_trait_item_const(self, item);
    }
}

impl Rule for TimePrimitive {
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
                let mut visitor = TimeVisitor {
                    context,
                    ordinal,
                    module: module.id,
                    spans: Vec::new(),
                    bindings: Vec::new(),
                };
                visitor.visit_file(&file.syntax);
                for span in visitor.spans {
                    findings.push(build_node_finding("TimePrimitive", &file.path, find_node_span(span),
                format_module_path(&index.crate_name, &file.module.path), "Keep Instant process-local; represent time constants as Duration (time-api.rs.md)".to_owned()));
                }
            }
        }
        findings
    }
}
