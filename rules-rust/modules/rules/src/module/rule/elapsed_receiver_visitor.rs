use crate::source::model::NodeSpan;
use crate::source::util::{find_node_span, parse_macro_arguments};
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct ElapsedReceiverVisitor {
    receivers: Vec<NodeSpan>,
}

impl ElapsedReceiverVisitor {
    pub(crate) fn collect(syntax: &syn::File) -> Vec<NodeSpan> {
        let mut visitor = Self {
            receivers: Vec::new(),
        };
        visitor.visit_file(syntax);
        visitor.receivers
    }
}

impl<'ast> Visit<'ast> for ElapsedReceiverVisitor {
    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        for argument in &parse_macro_arguments(item) {
            self.visit_expr(argument);
        }
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        if call.method == "elapsed" && matches!(&*call.receiver, syn::Expr::Path(_)) {
            self.receivers.push(find_node_span(call.receiver.span()));
        }
        syn::visit::visit_expr_method_call(self, call);
    }
}
