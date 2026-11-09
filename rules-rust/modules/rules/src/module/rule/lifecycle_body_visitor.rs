use std::collections::BTreeSet;
use syn::visit::Visit;

pub(crate) struct LifecycleBodyVisitor<'scope> {
    context: &'scope crate::check::model::RuleContext,
    ordinal: usize,
    module: usize,
    method: &'scope str,
    owned: &'scope [String],
    scopes: &'scope [String],
    pub(crate) invalid: bool,
    pub(crate) aborted: BTreeSet<String>,
    pub(crate) child_lifecycle: bool,
}

impl LifecycleBodyVisitor<'_> {
    pub(crate) fn inspect<'scope>(
        context: &'scope crate::check::model::RuleContext,
        ordinal: usize,
        module: usize,
        block: &syn::Block,
        method: &'scope str,
        owned: &'scope [String],
        scopes: &'scope [String],
    ) -> LifecycleBodyVisitor<'scope> {
        let mut visitor = LifecycleBodyVisitor {
            context,
            ordinal,
            module,
            method,
            owned,
            scopes,
            invalid: false,
            aborted: BTreeSet::new(),
            child_lifecycle: false,
        };
        visitor.visit_block(block);
        visitor
    }

    fn field(expression: &syn::Expr) -> Option<String> {
        if let syn::Expr::Reference(reference) = expression {
            return Self::field(&reference.expr);
        }
        let syn::Expr::Field(field) = expression else {
            return None;
        };
        if !matches!(field.base.as_ref(), syn::Expr::Path(path) if path.path.is_ident("self")) {
            return None;
        }
        match &field.member {
            syn::Member::Named(name) => Some(name.to_string()),
            _ => None,
        }
    }

    fn same_ufcs(&self, call: &syn::ExprCall) -> bool {
        let syn::Expr::Path(callee) = call.func.as_ref() else {
            return false;
        };
        if callee
            .path
            .segments
            .last()
            .is_none_or(|segment| segment.ident != self.method)
        {
            return false;
        }
        let mut path = callee.path.clone();
        path.segments.pop();
        path.segments.pop_punct();
        let receiver = call.args.first().and_then(Self::field);
        receiver.is_some_and(|field| self.owned.contains(&field))
            && crate::module::util::lifecycle_trait_path(
                self.context,
                self.ordinal,
                self.module,
                &path,
            )
    }
}

impl<'ast> Visit<'ast> for LifecycleBodyVisitor<'_> {
    fn visit_expr_await(&mut self, expression: &'ast syn::ExprAwait) {
        self.invalid |= self.method == "on_disable";
        syn::visit::visit_expr_await(self, expression);
    }

    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        let child = self.same_ufcs(call);
        self.invalid |= !child;
        self.child_lifecycle |= child;
        syn::visit::visit_expr_call(self, call);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        let name = call.method.to_string();
        let field = Self::field(&call.receiver);
        let abort = self.method == "on_disable"
            && name == "abort"
            && field
                .as_ref()
                .is_some_and(|field| self.scopes.contains(field));
        let child = name == self.method
            && field
                .as_ref()
                .is_some_and(|field| self.owned.contains(field));
        if abort {
            self.aborted.extend(field);
        }
        self.child_lifecycle |= child;
        self.invalid |= !abort && !child;
        syn::visit::visit_expr_method_call(self, call);
    }
}
