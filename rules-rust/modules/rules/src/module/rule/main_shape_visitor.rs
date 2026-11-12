use crate::check::model::RuleContext;
use crate::module::util::is_di_type;
use crate::runtime::rule::SelectBranches;
use crate::source::model::Resolution;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct MainShapeVisitor<'context> {
    context: &'context RuleContext,
    ordinal: usize,
    module: usize,
    runtime: bool,
    blocks: usize,
    in_block: bool,
    root: bool,
    enabled: bool,
    disabled_after: bool,
    roots: std::collections::BTreeSet<String>,
    runtimes: std::collections::BTreeSet<String>,
    signals: std::collections::BTreeSet<String>,
    owners: std::collections::BTreeMap<String, syn::Path>,
    builder_calls: usize,
    returned_roots: Vec<crate::source::model::NodeSpan>,
    shutdown_waited: bool,
    early_exit: bool,
    follows_helpers: bool,
}

impl MainShapeVisitor<'_> {
    fn new(context: &RuleContext, ordinal: usize, module: usize) -> MainShapeVisitor<'_> {
        MainShapeVisitor {
            context,
            ordinal,
            module,
            runtime: false,
            blocks: 0,
            in_block: false,
            root: false,
            enabled: false,
            disabled_after: false,
            roots: std::collections::BTreeSet::new(),
            runtimes: std::collections::BTreeSet::new(),
            signals: std::collections::BTreeSet::new(),
            owners: std::collections::BTreeMap::new(),
            builder_calls: 0,
            returned_roots: Vec::new(),
            shutdown_waited: false,
            early_exit: false,
            follows_helpers: true,
        }
    }

    fn build_flow(&self) -> Flow {
        Flow {
            enabled: self.enabled,
            disabled_after: self.disabled_after,
            shutdown_waited: self.shutdown_waited,
        }
    }

    fn restore(&mut self, flow: Flow) {
        self.enabled = flow.enabled;
        self.disabled_after = flow.disabled_after;
        self.shutdown_waited = flow.shutdown_waited;
    }

    fn find_tokio_segments(&self, path: &syn::Path) -> Vec<String> {
        match self.context.indexes[self.ordinal].resolve_path(self.module, path) {
            Resolution::External(path) => {
                let path = self.context.resolve_external_identity(self.ordinal, &path);
                if path.package == "tokio" {
                    path.segments
                } else {
                    Vec::new()
                }
            }
            _ => Vec::new(),
        }
    }

    fn is_cancellation_token_constructor(&self, call: &syn::ExprCall) -> bool {
        let syn::Expr::Path(path) = call.func.as_ref() else {
            return false;
        };
        self.context
            .resolve_external_call_paths(self.ordinal, self.module, &path.path)
            .iter()
            .any(|path| path.format_path() == "tokio_util::sync::CancellationToken::new")
    }

    fn is_signal_source(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::MethodCall(call) => self.is_signal_source(&call.receiver),
            syn::Expr::Try(attempt) => self.is_signal_source(&attempt.expr),
            syn::Expr::Call(call) => {
                let is_signal = matches!(call.func.as_ref(), syn::Expr::Path(path) if {
                    let segments = self.find_tokio_segments(&path.path);
                    segments.iter().any(|segment| segment == "signal")
                        && (segments.last().is_some_and(|segment| segment == "signal")
                            || segments.iter().any(|segment| segment == "windows"))
                });
                is_signal || self.is_cancellation_token_constructor(call)
            }
            _ => false,
        }
    }

    fn is_signal_wait(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Call(call) => {
                matches!(call.func.as_ref(), syn::Expr::Path(path) if self
                    .find_tokio_segments(&path.path)
                    .ends_with(&["signal".to_owned(), "ctrl_c".to_owned()]))
            }
            syn::Expr::MethodCall(call) => {
                matches!(call.method.to_string().as_str(), "recv" | "cancelled")
                    && matches!(call.receiver.as_ref(), syn::Expr::Path(path) if path
                        .path
                        .get_ident()
                        .is_some_and(|name| self.signals.contains(&name.to_string())))
            }
            _ => false,
        }
    }

    fn root_target(&self, target: &crate::source::model::WorkspaceResolvedPath) -> bool {
        target.path.item.is_some_and(|identity| {
            let item = &self.context.indexes[target.index].items[identity];
            item.name == self.context.root_module_name()
                && is_di_type(self.context, target.index, item)
        })
    }

    fn root_constructor(&self, expression: &syn::Expr) -> bool {
        let syn::Expr::Call(call) = expression else {
            return false;
        };
        let syn::Expr::Path(callee) = call.func.as_ref() else {
            return false;
        };
        if callee
            .path
            .segments
            .last()
            .is_none_or(|segment| segment.ident != "new")
        {
            return false;
        }
        let mut path = callee.path.clone();
        path.segments.pop();
        path.segments.pop_punct();
        self.context
            .resolve_workspace_path(self.ordinal, self.module, &path)
            .iter()
            .any(|target| self.root_target(target))
    }

    fn root_expression(&self, expression: &syn::Expr) -> bool {
        matches!(expression, syn::Expr::Path(path) if path.path.get_ident().is_some_and(|name| self.roots.contains(&name.to_string())))
            || self.root_constructor(expression)
    }

    fn binding(pattern: &syn::Pat) -> Option<&syn::Ident> {
        match pattern {
            syn::Pat::Ident(binding) => Some(&binding.ident),
            syn::Pat::Type(typed) => Self::binding(&typed.pat),
            _ => None,
        }
    }

    fn find_owner_path(local: &syn::Local) -> Option<syn::Path> {
        if let syn::Pat::Type(annotated) = &local.pat
            && let syn::Type::Path(typed) = annotated.ty.as_ref()
        {
            return Some(typed.path.clone());
        }
        let syn::Expr::Call(call) = local.init.as_ref()?.expr.as_ref() else {
            return None;
        };
        let syn::Expr::Path(callee) = call.func.as_ref() else {
            return None;
        };
        let mut owner = callee.path.clone();
        owner.segments.pop();
        owner.segments.pop_punct();
        (!owner.segments.is_empty()).then_some(owner)
    }

    fn waits_for_signal(&self, ordinal: usize, module: usize, block: &syn::Block) -> bool {
        let mut helper = Self::new(self.context, ordinal, module);
        helper.follows_helpers = false;
        helper.in_block = true;
        helper.enabled = true;
        helper.visit_block(block);
        helper.shutdown_waited
    }

    fn has_waiting_method(&self, owner: &syn::Path, name: &syn::Ident) -> bool {
        self.context
            .resolve_workspace_path(self.ordinal, self.module, owner)
            .iter()
            .any(|target| {
                let Some(identity) = target.path.item else {
                    return false;
                };
                let index = &self.context.indexes[target.index];
                index
                    .implementations
                    .iter()
                    .filter(|implementation| {
                        crate::module::util::owns_implementation(index, identity, implementation)
                    })
                    .any(|implementation| {
                        implementation.syntax.items.iter().any(|member| {
                            matches!(member, syn::ImplItem::Fn(method)
                            if method.sig.ident == *name
                                && method.sig.asyncness.is_some()
                                && self.waits_for_signal(
                                    target.index,
                                    implementation.module,
                                    &method.block,
                                ))
                        })
                    })
            })
    }

    fn is_associated_signal_helper(&self, path: &syn::Path) -> bool {
        let Some(name) = path.segments.last().map(|segment| &segment.ident) else {
            return false;
        };
        let mut owner = path.clone();
        owner.segments.pop();
        owner.segments.pop_punct();
        if owner.segments.is_empty() {
            return false;
        }
        self.has_waiting_method(&owner, name)
    }

    fn is_method_signal_helper(&self, call: &syn::ExprMethodCall) -> bool {
        let syn::Expr::Path(receiver) = call.receiver.as_ref() else {
            return false;
        };
        let Some(name) = receiver.path.get_ident() else {
            return false;
        };
        self.owners
            .get(&name.to_string())
            .is_some_and(|owner| self.has_waiting_method(owner, &call.method))
    }

    fn is_signal_helper(&self, expression: &syn::Expr) -> bool {
        if let syn::Expr::MethodCall(call) = expression {
            return self.follows_helpers && self.is_method_signal_helper(call);
        }
        let syn::Expr::Call(call) = expression else {
            return false;
        };
        let syn::Expr::Path(callee) = call.func.as_ref() else {
            return false;
        };
        if !self.follows_helpers {
            return false;
        }
        let is_free_helper = self
            .context
            .resolve_workspace_path(self.ordinal, self.module, &callee.path)
            .iter()
            .any(|target| {
                let Some(identity) = target.path.item else {
                    return false;
                };
                let item = &self.context.indexes[target.index].items[identity];
                matches!(&item.syntax, syn::Item::Fn(function)
                    if function.sig.asyncness.is_some()
                        && self.waits_for_signal(target.index, item.module, &function.block))
            });
        is_free_helper || self.is_associated_signal_helper(&callee.path)
    }

    pub(crate) fn valid(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        block: &syn::Block,
    ) -> bool {
        let mut visitor = Self::new(context, ordinal, module);
        visitor.visit_block(block);
        visitor.runtime
            && visitor.blocks == 1
            && visitor.root
            && visitor.enabled
            && visitor.disabled_after
            && visitor.shutdown_waited
            && !visitor.early_exit
    }
}

impl<'ast> Visit<'ast> for MainShapeVisitor<'_> {
    fn visit_expr_return(&mut self, returned: &'ast syn::ExprReturn) {
        self.early_exit |= self.enabled && (!self.shutdown_waited || !self.disabled_after);
        syn::visit::visit_expr_return(self, returned);
    }

    fn visit_expr_try(&mut self, attempt: &'ast syn::ExprTry) {
        syn::visit::visit_expr_try(self, attempt);
        self.early_exit |= self.enabled && (!self.shutdown_waited || !self.disabled_after);
    }

    fn visit_expr_await(&mut self, awaited: &'ast syn::ExprAwait) {
        if !(self.in_block && self.enabled) {
            syn::visit::visit_expr_await(self, awaited);
            return;
        }
        let waits_for_signal =
            self.is_signal_wait(&awaited.base) || self.is_signal_helper(&awaited.base);
        if waits_for_signal {
            self.shutdown_waited = true;
        }
        syn::visit::visit_expr_await(self, awaited);
    }

    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        let select = self
            .find_tokio_segments(&item.path)
            .last()
            .is_some_and(|segment| segment == "select");
        if !(self.in_block && self.enabled && select) {
            return;
        }
        let Ok(branches) = syn::parse2::<SelectBranches>(item.tokens.clone()) else {
            return;
        };
        if branches
            .futures
            .iter()
            .any(|future| self.is_signal_wait(future))
        {
            self.shutdown_waited = true;
        }
    }

    fn visit_expr_match(&mut self, branch: &'ast syn::ExprMatch) {
        self.visit_expr(&branch.expr);
        let before = self.build_flow();
        let mut merged: Option<Flow> = None;
        for arm in &branch.arms {
            self.restore(before);
            self.visit_arm(arm);
            let arm_flow = self.build_flow();
            merged = Some(merged.map_or(arm_flow, |flow| flow.both(arm_flow)));
        }
        self.restore(merged.unwrap_or(before));
    }

    fn visit_expr_if(&mut self, branch: &'ast syn::ExprIf) {
        self.visit_expr(&branch.cond);
        let literal = match branch.cond.as_ref() {
            syn::Expr::Lit(value) => match &value.lit {
                syn::Lit::Bool(value) => Some(value.value),
                _ => None,
            },
            _ => None,
        };
        if literal == Some(false) {
            if let Some((_, expression)) = &branch.else_branch {
                self.visit_expr(expression);
            }
            return;
        }
        let before = self.build_flow();
        self.visit_block(&branch.then_branch);
        if literal == Some(true) {
            return;
        }
        let then_flow = self.build_flow();
        self.restore(before);
        if let Some((_, expression)) = &branch.else_branch {
            self.visit_expr(expression);
        }
        let else_flow = self.build_flow();
        self.restore(then_flow.both(else_flow));
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        let builders = self.builder_calls;
        syn::visit::visit_local(self, local);
        let (Some(binding), Some(initializer)) = (Self::binding(&local.pat), &local.init) else {
            return;
        };
        let root = self.root_expression(&initializer.expr);
        let returned_root = match initializer.expr.as_ref() {
            syn::Expr::MethodCall(call) if call.method == "block_on" => match call.args.first() {
                Some(syn::Expr::Async(block)) => self
                    .returned_roots
                    .contains(&crate::source::util::find_node_span(block.block.span())),
                _ => false,
            },
            _ => false,
        };
        self.roots.remove(&binding.to_string());
        self.runtimes.remove(&binding.to_string());
        self.signals.remove(&binding.to_string());
        self.owners.remove(&binding.to_string());
        if self.is_signal_source(&initializer.expr) {
            self.signals.insert(binding.to_string());
        }
        if let Some(owner) = Self::find_owner_path(local) {
            self.owners.insert(binding.to_string(), owner);
        }
        if self.builder_calls > builders {
            self.runtimes.insert(binding.to_string());
        }
        if root || returned_root {
            self.roots.insert(binding.to_string());
        }
    }
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if let syn::Expr::Path(callee) = call.func.as_ref() {
            let drops = matches!(self.context.indexes[self.ordinal].resolve_path(self.module, &callee.path), Resolution::External(path) if path == "std::mem::drop" || path == "core::mem::drop")
                || callee.path.is_ident("drop");
            if drops
                && let Some(syn::Expr::Path(path)) = call.args.first()
                && let Some(name) = path.path.get_ident()
            {
                self.runtimes.remove(&name.to_string());
            }
            if let Resolution::External(path) =
                self.context.indexes[self.ordinal].resolve_path(self.module, &callee.path)
            {
                let path = self.context.resolve_external_identity(self.ordinal, &path);
                let builder = path.package == "tokio"
                    && path.segments.iter().any(|segment| segment == "runtime")
                    && path.segments.iter().any(|segment| segment == "Builder")
                    && path
                        .segments
                        .last()
                        .is_some_and(|segment| segment.starts_with("new_"));
                self.runtime |= builder;
                self.builder_calls += usize::from(builder);
            }
            let constructor = callee
                .path
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "new");
            if self.in_block && constructor {
                let mut path = callee.path.clone();
                path.segments.pop();
                path.segments.pop_punct();
                self.root |= self
                    .context
                    .resolve_workspace_path(self.ordinal, self.module, &path)
                    .iter()
                    .any(|target| self.root_target(target));
            }
        }
        syn::visit::visit_expr_call(self, call);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        let runtime = matches!(call.receiver.as_ref(), syn::Expr::Path(path) if path.path.get_ident().is_some_and(|name| self.runtimes.contains(&name.to_string())));
        if call.method == "block_on" && runtime && !self.in_block {
            self.blocks += 1;
            self.in_block = true;
            for argument in &call.args {
                self.visit_expr(argument);
            }
            self.in_block = false;
            return;
        }
        self.enabled |=
            self.in_block && self.root_expression(&call.receiver) && call.method == "on_enable";
        self.disabled_after |= !self.in_block
            && self.blocks > 0
            && !self.runtimes.is_empty()
            && self.root_expression(&call.receiver)
            && call.method == "on_disable";
        syn::visit::visit_expr_method_call(self, call);
    }

    fn visit_item_fn(&mut self, _: &'ast syn::ItemFn) {}
    fn visit_expr_closure(&mut self, _: &'ast syn::ExprClosure) {}

    fn visit_block(&mut self, block: &'ast syn::Block) {
        let roots = self.roots.clone();
        let runtimes = self.runtimes.clone();
        let signals = self.signals.clone();
        let owners = self.owners.clone();
        syn::visit::visit_block(self, block);
        if let Some(syn::Stmt::Expr(expression, None)) = block.stmts.last()
            && self.root_expression(expression)
        {
            self.returned_roots
                .push(crate::source::util::find_node_span(block.span()));
        }
        self.roots = roots;
        self.runtimes = runtimes;
        self.signals = signals;
        self.owners = owners;
    }
}

#[derive(Clone, Copy)]
struct Flow {
    enabled: bool,
    disabled_after: bool,
    shutdown_waited: bool,
}

impl Flow {
    fn both(self, other: Flow) -> Flow {
        Flow {
            enabled: self.enabled && other.enabled,
            disabled_after: self.disabled_after && other.disabled_after,
            shutdown_waited: self.shutdown_waited && other.shutdown_waited,
        }
    }
}
