use crate::check::model::RuleContext;
use crate::module::util::resolved_di_type;
use crate::source::util::find_node_span;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct RootConstructionVisitor<'context> {
    context: &'context RuleContext,
    ordinal: usize,
    module: usize,
    invalid: Vec<crate::source::model::NodeSpan>,
}

impl<'context> RootConstructionVisitor<'context> {
    fn invalid_call(&self, call: &syn::ExprCall) -> bool {
        let syn::Expr::Path(callee) = call.func.as_ref() else {
            return false;
        };
        let mut path = callee.path.clone();
        let constructor = path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "new");
        let qualified = path.segments.len() >= 2;
        if !qualified || !constructor {
            return false;
        }
        path.segments.pop();
        path.segments.pop_punct();
        let targets = self
            .context
            .resolve_workspace_path(self.ordinal, self.module, &path);
        targets.is_empty()
            || targets
                .iter()
                .any(|target| !resolved_di_type(self.context, target))
    }
    pub(crate) fn collect(
        context: &'context RuleContext,
        ordinal: usize,
        module: usize,
        block: &syn::Block,
    ) -> Vec<crate::source::model::NodeSpan> {
        let mut visitor = Self {
            context,
            ordinal,
            module,
            invalid: Vec::new(),
        };
        visitor.visit_block(block);
        visitor.invalid
    }
}

impl<'ast> Visit<'ast> for RootConstructionVisitor<'_> {
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if self.invalid_call(call) {
            self.invalid.push(find_node_span(call.span()));
        }
        syn::visit::visit_expr_call(self, call);
    }
}
