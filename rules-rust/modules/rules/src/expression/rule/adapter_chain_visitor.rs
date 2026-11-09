use crate::source::model::NodeSpan;
use crate::source::util::{find_node_span, parse_macro_arguments};
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct AdapterChainVisitor {
    parent_adapter: bool,
    spans: Vec<NodeSpan>,
}

impl AdapterChainVisitor {
    pub(crate) fn collect(syntax: &syn::File) -> Vec<NodeSpan> {
        let mut visitor = Self {
            parent_adapter: false,
            spans: Vec::new(),
        };
        visitor.visit_file(syntax);
        visitor.spans
    }

    fn adapter(call: &syn::ExprMethodCall) -> bool {
        [
            "map",
            "map_err",
            "and_then",
            "filter",
            "filter_map",
            "flat_map",
            "inspect",
            "inspect_err",
            "or_else",
            "map_while",
            "skip_while",
            "take_while",
        ]
        .iter()
        .any(|name| call.method == *name)
            && call.args.len() == 1
            && matches!(call.args.first(), Some(syn::Expr::Path(_)))
    }

    fn length(expression: &syn::Expr) -> usize {
        match expression {
            syn::Expr::MethodCall(call) if Self::adapter(call) => 1 + Self::length(&call.receiver),
            syn::Expr::Paren(paren) => Self::length(&paren.expr),
            syn::Expr::Group(group) => Self::length(&group.expr),
            _ => 0,
        }
    }
}

impl<'ast> Visit<'ast> for AdapterChainVisitor {
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        let adapter = Self::adapter(call);
        if adapter && !self.parent_adapter && Self::length(&call.receiver) >= 2 {
            self.spans.push(find_node_span(call.span()));
        }
        let parent = self.parent_adapter;
        self.parent_adapter = adapter;
        self.visit_expr(&call.receiver);
        self.parent_adapter = false;
        for argument in &call.args {
            self.visit_expr(argument);
        }
        self.parent_adapter = parent;
    }

    fn visit_macro(&mut self, expression: &'ast syn::Macro) {
        for argument in &parse_macro_arguments(expression) {
            self.visit_expr(argument);
        }
    }
}
