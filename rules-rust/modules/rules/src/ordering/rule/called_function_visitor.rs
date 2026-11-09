use crate::source::util::parse_macro_arguments;
use std::collections::HashSet;
use syn::visit::Visit;

pub(crate) struct CalledFunctionVisitor<'scope> {
    own_name: Option<&'scope str>,
    calls: HashSet<String>,
}

impl<'scope> CalledFunctionVisitor<'scope> {
    pub(crate) fn new(own_name: Option<&'scope str>) -> Self {
        Self {
            own_name,
            calls: HashSet::new(),
        }
    }

    fn calls_owner(path: &syn::ExprPath, name: &str) -> bool {
        let segments = &path.path.segments;
        match &path.qself {
            None => {
                if segments.len() != 2 {
                    return false;
                }
                segments[0].ident == "Self" || segments[0].ident == name
            }
            Some(self_type) => {
                if segments.len() != 1 || self_type.position != 0 {
                    return false;
                }
                matches!(&*self_type.ty, syn::Type::Path(value) if value.path.is_ident("Self") || value.path.is_ident(name))
            }
        }
    }

    pub(crate) fn collect_calls(mut self, block: &syn::Block) -> HashSet<String> {
        self.visit_block(block);
        self.calls
    }
}

impl<'ast> Visit<'ast> for CalledFunctionVisitor<'_> {
    fn visit_expr_call(&mut self, expression: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = &*expression.func {
            let segments = &path.path.segments;
            let free = self.own_name.is_none() && path.qself.is_none() && segments.len() == 1;
            let own = self
                .own_name
                .is_some_and(|name| Self::calls_owner(path, name));
            let called_name = segments
                .last()
                .filter(|_| free || own)
                .map(|segment| segment.ident.to_string());
            if let Some(name) = called_name {
                self.calls.insert(name);
            }
        }
        syn::visit::visit_expr_call(self, expression);
    }

    fn visit_expr_method_call(&mut self, expression: &'ast syn::ExprMethodCall) {
        let has_owner = self.own_name.is_some();
        let has_self_receiver =
            matches!(&*expression.receiver, syn::Expr::Path(path) if path.path.is_ident("self"));
        if has_owner && has_self_receiver {
            self.calls.insert(expression.method.to_string());
        }
        syn::visit::visit_expr_method_call(self, expression);
    }

    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        for argument in &parse_macro_arguments(item) {
            self.visit_expr(argument);
        }
    }
}
