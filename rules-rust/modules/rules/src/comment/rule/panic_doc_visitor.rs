use crate::check::model::RuleContext;
use crate::source::model::Resolution;
use std::collections::BTreeMap;
use syn::visit::Visit;

pub(crate) struct PanicDocVisitor<'context> {
    context: &'context RuleContext,
    ordinal: usize,
    module: usize,
    cfg: crate::source::model::CfgPredicate,
    parameters: BTreeMap<String, syn::Type>,
    found: bool,
}

impl PanicDocVisitor<'_> {
    pub(crate) fn contains_panic(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        signature: &syn::Signature,
        cfg: &crate::source::model::CfgPredicate,
        block: &syn::Block,
    ) -> bool {
        let parameters = signature
            .inputs
            .iter()
            .filter_map(|input| match input {
                syn::FnArg::Typed(parameter) => {
                    if let syn::Pat::Ident(binding) = parameter.pat.as_ref() {
                        Some((binding.ident.to_string(), (*parameter.ty).clone()))
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect();
        let mut visitor = PanicDocVisitor {
            context,
            ordinal,
            module,
            cfg: cfg.clone(),
            parameters,
            found: false,
        };
        visitor.visit_block(block);
        visitor.found
    }

    fn standard_receiver(&self, expression: &syn::Expr) -> bool {
        let syn::Expr::Path(path) = expression else {
            return false;
        };
        let Some(name) = path.path.get_ident() else {
            return false;
        };
        let Some(typed) = self.parameters.get(&name.to_string()) else {
            return false;
        };
        let workspace = self
            .context
            .resolve_outer_workspace_type(self.ordinal, self.module, typed)
            .iter()
            .any(|target| self.cfg.and(&target.path.cfg).is_satisfiable());
        !workspace
            && self
                .context
                .resolve_outer_external_type_paths_under(
                    self.ordinal,
                    self.module,
                    typed,
                    &self.cfg,
                )
                .iter()
                .any(|path| {
                    ["std", "core"].contains(&path.package.as_str())
                        && path
                            .segments
                            .last()
                            .is_some_and(|name| ["Option", "Result"].contains(&name.as_str()))
                })
    }
}

impl<'ast> Visit<'ast> for PanicDocVisitor<'_> {
    fn visit_macro(&mut self, expression: &'ast syn::Macro) {
        let builtin = expression.path.segments.last().is_some_and(|segment| {
            ["panic", "assert", "unreachable"]
                .iter()
                .any(|name| segment.ident == *name)
        });
        let qualified = expression
            .path
            .segments
            .first()
            .is_some_and(|segment| segment.ident == "std" || segment.ident == "core");
        let prelude = expression.path.segments.len() == 1
            && !matches!(
                self.context.indexes[self.ordinal].resolve_path(self.module, &expression.path),
                Resolution::Internal(_)
            );
        self.found |= builtin && (qualified || prelude);
        for argument in &crate::source::util::parse_macro_arguments(expression) {
            self.visit_expr(argument);
        }
    }

    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = call.func.as_ref() {
            self.found |= self
                .context
                .resolve_external_call_paths(self.ordinal, self.module, &path.path)
                .iter()
                .any(|path| {
                    ["std", "core"].contains(&path.package.as_str())
                        && path
                            .segments
                            .iter()
                            .any(|name| ["Option", "Result"].contains(&name.as_str()))
                        && path
                            .segments
                            .last()
                            .is_some_and(|name| ["unwrap", "expect"].contains(&name.as_str()))
                });
        }
        syn::visit::visit_expr_call(self, call);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        self.found |= (call.method == "unwrap" || call.method == "expect")
            && self.standard_receiver(&call.receiver);
        syn::visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_closure(&mut self, _: &'ast syn::ExprClosure) {}
    fn visit_expr_async(&mut self, _: &'ast syn::ExprAsync) {}
    fn visit_item_fn(&mut self, _: &'ast syn::ItemFn) {}
}
