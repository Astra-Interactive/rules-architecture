use crate::check::model::RuleContext;
use crate::module::util::is_di_type;
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
    builder_calls: usize,
    returned_roots: Vec<crate::source::model::NodeSpan>,
    shutdown_waited: bool,
    early_exit: bool,
}

impl MainShapeVisitor<'_> {
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

    pub(crate) fn valid(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        block: &syn::Block,
    ) -> bool {
        let mut visitor = MainShapeVisitor {
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
            builder_calls: 0,
            returned_roots: Vec::new(),
            shutdown_waited: false,
            early_exit: false,
        };
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
        self.early_exit |= self.enabled && !self.shutdown_waited;
        syn::visit::visit_expr_return(self, returned);
    }

    fn visit_expr_await(&mut self, awaited: &'ast syn::ExprAwait) {
        if self.in_block
            && self.enabled
            && let syn::Expr::Call(call) = awaited.base.as_ref()
            && let syn::Expr::Path(path) = call.func.as_ref()
        {
            let signal =
                match self.context.indexes[self.ordinal].resolve_path(self.module, &path.path) {
                    Resolution::External(path) => {
                        let path = self.context.resolve_external_identity(self.ordinal, &path);
                        path.package == "tokio"
                            && path
                                .segments
                                .ends_with(&["signal".to_owned(), "ctrl_c".to_owned()])
                    }
                    _ => false,
                };
            self.shutdown_waited |= signal;
        }
        syn::visit::visit_expr_await(self, awaited);
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
        let waited_before = self.shutdown_waited;
        self.visit_block(&branch.then_branch);
        if literal == Some(true) {
            return;
        }
        let waited_then = self.shutdown_waited;
        self.shutdown_waited = waited_before;
        if let Some((_, expression)) = &branch.else_branch {
            self.visit_expr(expression);
        }
        self.shutdown_waited &= waited_then;
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
        syn::visit::visit_block(self, block);
        if let Some(syn::Stmt::Expr(expression, None)) = block.stmts.last()
            && self.root_expression(expression)
        {
            self.returned_roots
                .push(crate::source::util::find_node_span(block.span()));
        }
        self.roots = roots;
        self.runtimes = runtimes;
    }
}
