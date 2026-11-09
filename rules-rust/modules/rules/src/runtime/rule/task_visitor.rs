use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::{build_node_finding, format_module_path};
use crate::declaration::util::is_technology;
use crate::runtime::rule::ClosureEffects;
use crate::runtime::util::expects_lint;
use crate::runtime::util::node_predicate;
use crate::source::model::{Resolution, SourceFile};
use crate::source::util::{find_node_span, parse_macro_arguments};
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct TaskVisitor<'source> {
    context: &'source RuleContext,
    ordinal: usize,
    module: usize,
    file: &'source SourceFile,
    rule: &'static str,
    findings: Vec<RuleFinding>,
    async_context: bool,
    drop_context: bool,
    callback_depth: usize,
    discard: bool,
    thread_expect: bool,
    handles: Vec<(String, proc_macro2::Span, bool)>,
    local_self: bool,
    closures: Vec<(String, Option<syn::ExprClosure>, usize)>,
    bindings: Vec<(String, Option<usize>)>,
    return_start: usize,
}

impl TaskVisitor<'_> {
    pub(crate) fn check_context(context: &RuleContext, rule: &'static str) -> Vec<RuleFinding> {
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
                let mut visitor = TaskVisitor {
                    context,
                    ordinal,
                    module: module.id,
                    file,
                    rule,
                    findings: Vec::new(),
                    async_context: false,
                    drop_context: false,
                    callback_depth: 0,
                    discard: false,
                    thread_expect: expects_lint(&file.syntax.attrs, "disallowed_methods"),
                    handles: Vec::new(),
                    local_self: false,
                    closures: Vec::new(),
                    bindings: Vec::new(),
                    return_start: 0,
                };
                visitor.visit_file(&file.syntax);
                findings.extend(visitor.findings);
            }
        }
        findings
    }

    fn report(&mut self, span: proc_macro2::Span) {
        let index = &self.context.indexes[self.ordinal];
        if !node_predicate(index, self.module, &self.file.path, find_node_span(span))
            .is_satisfiable()
        {
            return;
        }
        if self.findings.iter().any(|finding| matches!(&finding.anchor, crate::check::model::Anchor::Node { span: existing, .. } if *existing == find_node_span(span))) { return; }
        self.findings.push(build_node_finding(self.rule, &self.file.path, find_node_span(span),
            format_module_path(&index.crate_name, &index.modules[self.module].module.path),
            format!("Follow the {} async ownership/context requirement (async.rs.md; no-callbacks.rs.md)", self.rule)));
    }

    fn external(&self, path: &syn::Path) -> Option<String> {
        let Resolution::External(path) =
            self.context.indexes[self.ordinal].resolve_path(self.module, path)
        else {
            return None;
        };
        Some(
            self.context
                .resolve_external_identity(self.ordinal, &path)
                .format_path(),
        )
    }

    fn is_spawn(&self, expression: &syn::Expr) -> bool {
        matches!(expression, syn::Expr::Call(call) if matches!(&*call.func, syn::Expr::Path(path)
            if self.external(&path.path).as_deref() == Some("tokio::spawn")))
    }

    fn check_unwind(&mut self, expression: &syn::ExprCall, canonical: Option<&str>) {
        if self.rule != "CatchUnwind"
            || !canonical.is_some_and(|path| {
                matches!(
                    path,
                    "std::panic::catch_unwind" | "core::panic::catch_unwind"
                )
            })
        {
            return;
        }
        let calls_local = expression.args.iter().map(ClosureEffects::collect).any(|effects| effects.inline_logic
            || self.local_self && effects.methods.iter().any(|call| matches!(&*call.receiver, syn::Expr::Path(path) if path.path.is_ident("self")))
            || effects.calls.iter().any(|call| matches!(&*call.func, syn::Expr::Path(path)
                if matches!(self.context.indexes[self.ordinal].resolve_path(self.module, &path.path), Resolution::Internal(_)))));
        if calls_local {
            self.report(expression.span());
        }
    }

    fn constructed_type(expression: &syn::Expr) -> Option<syn::Type> {
        let syn::Expr::Call(call) = expression else {
            return None;
        };
        let syn::Expr::Path(path) = &*call.func else {
            return None;
        };
        if path.path.segments.len() < 2 {
            return None;
        }
        let mut owner = path.path.clone();
        owner.segments.pop();
        owner.segments.pop_punct();
        Some(syn::Type::Path(syn::TypePath {
            attrs: Vec::new(),
            qself: path.qself.clone(),
            path: owner,
        }))
    }

    fn discarded_guard(&self, expression: &syn::Expr) -> bool {
        let Some(typed) = Self::constructed_type(expression) else {
            return false;
        };
        self.context
            .resolve_workspace_type(self.ordinal, self.module, &typed)
            .iter()
            .any(|target| {
                let Some(identity) = target.path.item else {
                    return false;
                };
                let index = &self.context.indexes[target.index];
                let syn::Expr::Call(call) = expression else {
                    return false;
                };
                let syn::Expr::Path(path) = &*call.func else {
                    return false;
                };
                let Some(method) = path.path.segments.last() else {
                    return false;
                };
                let constructors =
                    self.guard_constructors(target.index, identity, &method.ident.to_string());
                if constructors.is_empty() {
                    return false;
                }
                index.implementations.iter().any(|implementation| {
                    let Some((path, _)) = &implementation.syntax.trait_ else {
                        return false;
                    };
                    let is_drop = self
                        .context
                        .resolve_external_call_paths(target.index, implementation.module, path)
                        .iter()
                        .any(|identity| {
                            matches!(
                                identity.format_path().as_str(),
                                "core::ops::Drop" | "std::ops::Drop"
                            )
                        });
                    is_drop
                        && constructors.iter().any(|constructor| {
                            target
                                .path
                                .cfg
                                .and(constructor)
                                .and(&implementation.cfg)
                                .is_satisfiable()
                        })
                        && index
                            .resolve_type_items(
                                implementation.module,
                                &implementation.syntax.self_ty,
                            )
                            .contains(&identity)
                })
            })
    }

    fn guard_constructors(
        &self,
        ordinal: usize,
        identity: usize,
        name: &str,
    ) -> Vec<crate::source::model::CfgPredicate> {
        let index = &self.context.indexes[ordinal];
        let mut constructors = Vec::new();
        for implementation in &index.implementations {
            if !index
                .resolve_type_items(implementation.module, &implementation.syntax.self_ty)
                .contains(&identity)
            {
                continue;
            }
            for item in &implementation.syntax.items {
                let syn::ImplItem::Fn(function) = item else {
                    continue;
                };
                if function.sig.receiver().is_some() || function.sig.ident != name {
                    continue;
                }
                let syn::ReturnType::Type(_, returned) = &function.sig.output else {
                    continue;
                };
                let cfg =
                    implementation
                        .cfg
                        .and(&crate::source::model::CfgPredicate::from_attributes(
                            &function.attrs,
                        ));
                constructors.extend(self.owned_return_configs(
                    ordinal,
                    implementation.module,
                    returned,
                    identity,
                    &cfg,
                ));
            }
        }
        constructors
    }

    fn owned_return_configs(
        &self,
        ordinal: usize,
        module: usize,
        returned: &syn::Type,
        identity: usize,
        cfg: &crate::source::model::CfgPredicate,
    ) -> Vec<crate::source::model::CfgPredicate> {
        self.context
            .resolve_type_syntax(ordinal, module, returned)
            .iter()
            .filter_map(|resolved| {
                let syn::Type::Path(path) = &resolved.typed else {
                    return None;
                };
                let own = path.path.is_ident("Self")
                    || self
                        .context
                        .resolve_outer_workspace_type(
                            resolved.index,
                            resolved.module,
                            &resolved.typed,
                        )
                        .iter()
                        .any(|target| {
                            target.index == ordinal && target.path.item == Some(identity)
                        });
                own.then(|| cfg.and(&resolved.cfg))
            })
            .collect()
    }

    fn resolve_handle(&self, expression: &syn::Expr) -> Option<usize> {
        let syn::Expr::Path(path) = expression else {
            return None;
        };
        self.bindings
            .iter()
            .rev()
            .find(|(name, _)| path.path.is_ident(name))
            .and_then(|(_, identity)| *identity)
    }

    fn consume_handle(&mut self, expression: &syn::Expr) {
        if let Some(identity) = self.resolve_handle(expression) {
            self.handles[identity].2 = true;
        }
    }

    fn shadow_pattern(&mut self, pattern: &syn::Pat) {
        match pattern {
            syn::Pat::Ident(name) => {
                self.bindings.push((name.ident.to_string(), None));
                let environment = self.closures.len();
                self.closures
                    .push((name.ident.to_string(), None, environment));
                if let Some((_, pattern)) = &name.subpat {
                    self.shadow_pattern(pattern);
                }
            }
            syn::Pat::Type(pattern) => self.shadow_pattern(&pattern.pat),
            syn::Pat::Paren(pattern) => self.shadow_pattern(&pattern.pat),
            syn::Pat::Reference(pattern) => self.shadow_pattern(&pattern.pat),
            syn::Pat::Tuple(pattern) => {
                for pattern in &pattern.elems {
                    self.shadow_pattern(pattern);
                }
            }
            syn::Pat::TupleStruct(pattern) => {
                for pattern in &pattern.elems {
                    self.shadow_pattern(pattern);
                }
            }
            syn::Pat::Slice(pattern) => {
                for pattern in &pattern.elems {
                    self.shadow_pattern(pattern);
                }
            }
            syn::Pat::Struct(pattern) => {
                for field in &pattern.fields {
                    self.shadow_pattern(&field.pat);
                }
            }
            syn::Pat::Or(pattern) => {
                for pattern in &pattern.cases {
                    self.shadow_pattern(pattern);
                }
            }
            _ => syn::visit::visit_pat(self, pattern),
        }
    }

    fn pattern_name(pattern: &syn::Pat) -> Option<&syn::Ident> {
        match pattern {
            syn::Pat::Ident(name) => Some(&name.ident),
            syn::Pat::Type(pattern) => Self::pattern_name(&pattern.pat),
            syn::Pat::Paren(pattern) => Self::pattern_name(&pattern.pat),
            _ => None,
        }
    }

    fn bind_local(&mut self, local: &syn::Local) {
        let alias = local
            .init
            .as_ref()
            .and_then(|init| self.resolve_handle(&init.expr));
        let handle = if self.rule == "DetachedTask"
            && Self::pattern_name(&local.pat).is_some()
            && local
                .init
                .as_ref()
                .is_some_and(|init| self.is_spawn(&init.expr))
        {
            let identity = self.handles.len();
            self.handles.push((String::new(), local.span(), false));
            Some(identity)
        } else {
            alias
        };
        self.shadow_pattern(&local.pat);
        let Some(name) = Self::pattern_name(&local.pat) else {
            return;
        };
        if let Some((_, identity)) = self
            .bindings
            .iter_mut()
            .rev()
            .find(|(binding, _)| *name == binding)
        {
            *identity = handle;
        }
        if self.rule == "CallbackBridgeShape" {
            let closure = local.init.as_ref().and_then(|init| match &*init.expr {
                syn::Expr::Closure(closure) => Some(closure.clone()),
                _ => None,
            });
            if let Some((_, bound, _)) = self
                .closures
                .iter_mut()
                .rev()
                .find(|(binding, _, _)| *name == binding)
            {
                *bound = closure;
            }
        }
    }

    fn visit_condition(&mut self, condition: &syn::Expr) {
        match condition {
            syn::Expr::Let(condition) => {
                self.visit_expr(&condition.expr);
                self.shadow_pattern(&condition.pat);
            }
            syn::Expr::Binary(binary) if matches!(binary.op, syn::BinOp::And(_)) => {
                self.visit_condition(&binary.left);
                self.visit_condition(&binary.right);
            }
            _ => self.visit_expr(condition),
        }
    }

    fn visit_callback_argument(&mut self, argument: &syn::Expr) {
        let bound = if let syn::Expr::Path(path) = argument {
            self.closures
                .iter()
                .rev()
                .find(|(name, _, _)| path.path.is_ident(name))
                .and_then(|(_, closure, environment)| {
                    closure.clone().map(|closure| (closure, *environment))
                })
        } else {
            None
        };
        if let Some((closure, environment)) = bound {
            let later_bindings = self.closures.split_off(environment);
            self.callback_depth += 1;
            self.visit_expr_closure(&closure);
            self.callback_depth -= 1;
            self.closures.truncate(environment);
            self.closures.extend(later_bindings);
        }
    }

    fn consume_argument(&mut self, argument: &syn::Expr, is_drop: bool) {
        if self.rule != "DetachedTask" {
            return;
        }
        if is_drop {
            let span = self.resolve_handle(argument).and_then(|identity| {
                let (_, span, consumed) = self.handles[identity];
                (!consumed).then_some(span)
            });
            if let Some(span) = span {
                self.report(span);
            }
        }
        self.consume_handle(argument);
    }
}

impl<'ast> Visit<'ast> for TaskVisitor<'_> {
    fn visit_pat_ident(&mut self, pattern: &'ast syn::PatIdent) {
        self.shadow_pattern(&syn::Pat::Ident(pattern.clone()));
    }
    fn visit_expr_if(&mut self, expression: &'ast syn::ExprIf) {
        let bindings = self.bindings.len();
        let closures = self.closures.len();
        self.visit_condition(&expression.cond);
        if let syn::Expr::Lit(literal) = &*expression.cond
            && let syn::Lit::Bool(value) = &literal.lit
        {
            if value.value {
                self.visit_block(&expression.then_branch);
            } else if let Some((_, otherwise)) = &expression.else_branch {
                self.visit_expr(otherwise);
            }
            self.bindings.truncate(bindings);
            self.closures.truncate(closures);
            return;
        }
        let handles = self.handles.clone();
        self.visit_block(&expression.then_branch);
        let then_handles = self.handles.clone();
        self.handles = handles;
        self.bindings.truncate(bindings);
        self.closures.truncate(closures);
        if let Some((_, otherwise)) = &expression.else_branch {
            self.visit_expr(otherwise);
        }
        for (handle, then_handle) in self.handles.iter_mut().zip(then_handles) {
            handle.2 &= then_handle.2;
        }
    }
    fn visit_block(&mut self, block: &'ast syn::Block) {
        let closures = self.closures.len();
        let bindings = self.bindings.len();
        let length = self.handles.len();
        syn::visit::visit_block(self, block);
        if let Some(syn::Stmt::Expr(value, None)) = block.stmts.last() {
            self.consume_handle(value);
        }
        let abandoned = self
            .handles
            .drain(length..)
            .filter_map(|(_, span, consumed)| (!consumed).then_some(span))
            .collect::<Vec<_>>();
        if self.rule == "DetachedTask" {
            for span in abandoned {
                self.report(span);
            }
        }
        self.closures.truncate(closures);
        self.bindings.truncate(bindings);
    }

    fn visit_expr_for_loop(&mut self, expression: &'ast syn::ExprForLoop) {
        self.visit_expr(&expression.expr);
        let bindings = self.bindings.len();
        let closures = self.closures.len();
        let handles = self.handles.clone();
        self.shadow_pattern(&expression.pat);
        self.visit_block(&expression.body);
        self.bindings.truncate(bindings);
        self.closures.truncate(closures);
        self.handles = handles;
    }

    fn visit_expr_while(&mut self, expression: &'ast syn::ExprWhile) {
        let bindings = self.bindings.len();
        let closures = self.closures.len();
        let handles = self.handles.clone();
        self.visit_condition(&expression.cond);
        self.visit_block(&expression.body);
        self.bindings.truncate(bindings);
        self.closures.truncate(closures);
        self.handles = handles;
    }

    fn visit_expr_match(&mut self, expression: &'ast syn::ExprMatch) {
        self.visit_expr(&expression.expr);
        let handles = self.handles.clone();
        let mut consumed = vec![true; handles.len()];
        for arm in &expression.arms {
            self.handles = handles.clone();
            self.visit_arm(arm);
            for (consumed, handle) in consumed.iter_mut().zip(&self.handles) {
                *consumed &= handle.2;
            }
        }
        self.handles = handles;
        for (handle, consumed) in self.handles.iter_mut().zip(consumed) {
            handle.2 = consumed;
        }
    }

    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        let bindings = self.bindings.len();
        let closures = self.closures.len();
        self.shadow_pattern(&arm.pat);
        self.visit_expr(&arm.body);
        self.bindings.truncate(bindings);
        self.closures.truncate(closures);
    }

    fn visit_expr_await(&mut self, expression: &'ast syn::ExprAwait) {
        self.consume_handle(&expression.base);
        syn::visit::visit_expr_await(self, expression);
    }

    fn visit_expr_return(&mut self, expression: &'ast syn::ExprReturn) {
        if let Some(value) = &expression.expr {
            self.consume_handle(value);
        }
        if self.rule == "DetachedTask" {
            let pending = self
                .handles
                .iter()
                .skip(self.return_start)
                .filter_map(|(_, span, consumed)| (!consumed).then_some(*span))
                .collect::<Vec<_>>();
            for span in pending {
                self.report(span);
            }
        }
        syn::visit::visit_expr_return(self, expression);
    }

    fn visit_expr_struct(&mut self, expression: &'ast syn::ExprStruct) {
        for field in &expression.fields {
            self.consume_handle(&field.expr);
        }
        syn::visit::visit_expr_struct(self, expression);
    }
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        let expectation = self.thread_expect;
        self.thread_expect |= expects_lint(&item.attrs, "disallowed_methods");
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
        self.thread_expect = expectation;
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        let return_start = self.return_start;
        self.return_start = 0;
        let handles = std::mem::take(&mut self.handles);
        let closures = std::mem::take(&mut self.closures);
        let bindings = std::mem::take(&mut self.bindings);
        let expectation = self.thread_expect;
        self.thread_expect |= expects_lint(&item.attrs, "disallowed_methods");
        let parent = (self.async_context, self.drop_context, self.callback_depth);
        self.async_context = item.sig.asyncness.is_some();
        self.drop_context = false;
        self.callback_depth = 0;
        syn::visit::visit_item_fn(self, item);
        (self.async_context, self.drop_context, self.callback_depth) = parent;
        self.thread_expect = expectation;
        self.handles = handles;
        self.closures = closures;
        self.bindings = bindings;
        self.return_start = return_start;
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let local_self = self.local_self;
        self.local_self = !self.context.indexes[self.ordinal]
            .resolve_type_items(self.module, &item.self_ty)
            .is_empty();
        let expectation = self.thread_expect;
        self.thread_expect |= expects_lint(&item.attrs, "disallowed_methods");
        let parent = self.drop_context;
        self.drop_context = item.trait_.as_ref().is_some_and(|(path, _)| {
            self.external(path).as_deref() == Some("std::ops::Drop")
                || self.external(path).as_deref() == Some("core::ops::Drop")
        });
        syn::visit::visit_item_impl(self, item);
        self.drop_context = parent;
        self.thread_expect = expectation;
        self.local_self = local_self;
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        let return_start = self.return_start;
        self.return_start = 0;
        let handles = std::mem::take(&mut self.handles);
        let bindings = std::mem::take(&mut self.bindings);
        let closures = std::mem::take(&mut self.closures);
        let callback = self.callback_depth;
        self.callback_depth = 0;
        let expectation = self.thread_expect;
        self.thread_expect |= expects_lint(&item.attrs, "disallowed_methods");
        let parent = self.async_context;
        self.async_context = item.sig.asyncness.is_some();
        let parent_drop = self.drop_context;
        self.drop_context &= item.sig.ident == "drop";
        syn::visit::visit_impl_item_fn(self, item);
        self.async_context = parent;
        self.drop_context = parent_drop;
        self.thread_expect = expectation;
        self.callback_depth = callback;
        self.handles = handles;
        self.bindings = bindings;
        self.closures = closures;
        self.return_start = return_start;
    }

    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        let parent = (
            self.async_context,
            self.drop_context,
            self.callback_depth,
            self.thread_expect,
        );
        self.async_context = item.sig.asyncness.is_some();
        self.drop_context = false;
        self.callback_depth = 0;
        self.thread_expect |= expects_lint(&item.attrs, "disallowed_methods");
        syn::visit::visit_trait_item_fn(self, item);
        (
            self.async_context,
            self.drop_context,
            self.callback_depth,
            self.thread_expect,
        ) = parent;
    }

    fn visit_expr_async(&mut self, expression: &'ast syn::ExprAsync) {
        let parent = self.async_context;
        let return_start = self.return_start;
        self.return_start = self.handles.len();
        self.async_context = true;
        syn::visit::visit_expr_async(self, expression);
        self.async_context = parent;
        self.return_start = return_start;
    }

    fn visit_expr_closure(&mut self, expression: &'ast syn::ExprClosure) {
        let return_start = self.return_start;
        self.return_start = self.handles.len();
        let bindings = self.bindings.len();
        let closures = self.closures.len();
        for parameter in &expression.inputs {
            self.shadow_pattern(parameter);
        }
        let parent = self.async_context;
        self.async_context = expression.asyncness.is_some();
        syn::visit::visit_expr_closure(self, expression);
        self.async_context = parent;
        self.bindings.truncate(bindings);
        self.closures.truncate(closures);
        self.return_start = return_start;
    }

    fn visit_stmt(&mut self, statement: &'ast syn::Stmt) {
        let parent = self.discard;
        self.discard = matches!(statement, syn::Stmt::Expr(_, Some(_)));
        syn::visit::visit_stmt(self, statement);
        self.discard = parent;
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        let expectation = self.thread_expect;
        self.thread_expect |= expects_lint(&local.attrs, "disallowed_methods");
        let parent = self.discard;
        self.discard = matches!(local.pat, syn::Pat::Wild(_));
        if let Some(init) = &local.init {
            if self.rule == "CallbackBridgeShape"
                && self.discard
                && self.discarded_guard(&init.expr)
            {
                self.report(local.span());
            }
            self.visit_expr(&init.expr);
            if let Some((_, diverge)) = &init.diverge {
                self.visit_expr(diverge);
            }
        }
        self.discard = parent;
        self.thread_expect = expectation;
        self.bind_local(local);
    }

    fn visit_expr_call(&mut self, expression: &'ast syn::ExprCall) {
        let expectation = self.thread_expect;
        self.thread_expect |= expects_lint(&expression.attrs, "disallowed_methods");
        let canonical = match &*expression.func {
            syn::Expr::Path(path) => self.external(&path.path),
            _ => None,
        };
        let spawn = canonical.as_deref() == Some("tokio::spawn");
        let violation = match self.rule {
            "ThreadPrimitive" => {
                self.thread_expect
                    && canonical.as_deref() == Some("std::thread::spawn")
                    && !is_technology(
                        self.context,
                        &self.context.indexes[self.ordinal],
                        self.module,
                    )
            }
            "DetachedTask" => spawn && self.discard,
            "BridgeCleanup" => spawn && self.drop_context,
            "CallbackBridgeShape" => spawn && self.callback_depth > 0,
            "BlockingCallInAsync" => {
                self.async_context
                    && canonical.as_deref().is_some_and(|path| {
                        path == "std::thread::sleep"
                            || path.starts_with("std::fs::")
                            || path.starts_with("std::sync::mpsc::")
                    })
            }
            _ => false,
        };
        if violation {
            self.report(expression.span());
        }
        self.check_unwind(expression, canonical.as_deref());
        let is_drop = canonical
            .as_deref()
            .is_some_and(|path| matches!(path, "std::mem::drop" | "core::mem::drop"));
        for argument in &expression.args {
            if self.rule == "CallbackBridgeShape" {
                self.visit_callback_argument(argument);
            }
            self.consume_argument(argument, is_drop);
            if self.rule == "DetachedTask" && is_drop && self.is_spawn(argument) {
                self.report(argument.span());
                let parent = self.discard;
                self.discard = false;
                self.visit_expr(argument);
                self.discard = parent;
            } else {
                let callback = matches!(argument, syn::Expr::Closure(_));
                self.callback_depth += usize::from(callback);
                let parent = self.discard;
                self.discard = false;
                self.visit_expr(argument);
                self.discard = parent;
                self.callback_depth -= usize::from(callback);
            }
        }
        self.visit_expr(&expression.func);
        self.thread_expect = expectation;
    }

    fn visit_expr_method_call(&mut self, expression: &'ast syn::ExprMethodCall) {
        if expression.method == "abort" {
            self.consume_handle(&expression.receiver);
        }
        let discard = self.discard;
        self.discard = false;
        self.visit_expr(&expression.receiver);
        for argument in &expression.args {
            if self.rule == "CallbackBridgeShape" {
                self.visit_callback_argument(argument);
            }
            self.consume_handle(argument);
            let callback = matches!(argument, syn::Expr::Closure(_));
            self.callback_depth += usize::from(callback);
            self.visit_expr(argument);
            self.callback_depth -= usize::from(callback);
        }
        self.discard = discard;
    }

    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        for argument in &parse_macro_arguments(item) {
            self.visit_expr(argument);
        }
    }
}
