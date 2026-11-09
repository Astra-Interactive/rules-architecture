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
        let calls_local = expression.args.iter().flat_map(|argument| ClosureEffects::collect(argument).calls).any(|call|
            matches!(&*call.func, syn::Expr::Path(path)
                if matches!(self.context.indexes[self.ordinal].resolve_path(self.module, &path.path), Resolution::Internal(_))));
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
        if path
            .path
            .segments
            .last()
            .is_none_or(|segment| segment.ident != "new")
        {
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
                let constructors = self.guard_constructors(target.index, identity);
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
                if function.sig.ident != "new" || function.sig.receiver().is_some() {
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
}

impl<'ast> Visit<'ast> for TaskVisitor<'_> {
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
        let expectation = self.thread_expect;
        self.thread_expect |= expects_lint(&item.attrs, "disallowed_methods");
        let parent = (self.async_context, self.drop_context, self.callback_depth);
        self.async_context = item.sig.asyncness.is_some();
        self.drop_context = false;
        self.callback_depth = 0;
        syn::visit::visit_item_fn(self, item);
        (self.async_context, self.drop_context, self.callback_depth) = parent;
        self.thread_expect = expectation;
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
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
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
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
        self.async_context = true;
        syn::visit::visit_expr_async(self, expression);
        self.async_context = parent;
    }

    fn visit_expr_closure(&mut self, expression: &'ast syn::ExprClosure) {
        let parent = self.async_context;
        self.async_context = expression.asyncness.is_some();
        syn::visit::visit_expr_closure(self, expression);
        self.async_context = parent;
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
        let discard = self.discard;
        self.discard = false;
        self.visit_expr(&expression.receiver);
        for argument in &expression.args {
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
