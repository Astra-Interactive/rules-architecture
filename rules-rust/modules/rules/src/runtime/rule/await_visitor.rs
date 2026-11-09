use syn::visit::Visit;

pub(crate) struct AwaitVisitor {
    found: bool,
}

impl AwaitVisitor {
    pub(crate) fn in_expression(expression: &syn::Expr) -> bool {
        let mut visitor = Self { found: false };
        if let syn::Expr::Async(future) = expression {
            visitor.visit_block(&future.block);
        } else {
            visitor.visit_expr(expression);
        }
        visitor.found
    }

    pub(crate) fn in_statement(statement: &syn::Stmt) -> bool {
        let mut visitor = Self { found: false };
        visitor.visit_stmt(statement);
        visitor.found
    }
}

impl<'ast> Visit<'ast> for AwaitVisitor {
    fn visit_expr_await(&mut self, _: &'ast syn::ExprAwait) {
        self.found = true;
    }
    fn visit_expr_async(&mut self, _: &'ast syn::ExprAsync) {}
    fn visit_expr_closure(&mut self, _: &'ast syn::ExprClosure) {}
    fn visit_item_fn(&mut self, _: &'ast syn::ItemFn) {}
}
