use syn::visit::Visit;

pub(crate) struct GuardWrites<'name> {
    name: &'name str,
    collections: &'name [String],
    pub(crate) writes: usize,
    pub(crate) trailing: bool,
}

impl GuardWrites<'_> {
    fn rooted(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Path(path) => path.path.is_ident(self.name),
            syn::Expr::Field(field) => self.rooted(&field.base),
            syn::Expr::Index(index) => self.rooted(&index.expr),
            syn::Expr::Unary(unary) => self.rooted(&unary.expr),
            syn::Expr::Paren(paren) => self.rooted(&paren.expr),
            _ => false,
        }
    }

    fn collection_path(&self, expression: &syn::Expr) -> Option<String> {
        match expression {
            syn::Expr::Path(path) if path.path.is_ident(self.name) => Some(String::new()),
            syn::Expr::Field(field) => self.collection_path(&field.base).map(|parent| {
                let name = match &field.member {
                    syn::Member::Named(name) => name.to_string(),
                    syn::Member::Unnamed(index) => index.index.to_string(),
                };
                if parent.is_empty() {
                    name
                } else {
                    format!("{parent}.{name}")
                }
            }),
            syn::Expr::Unary(unary) => self.collection_path(&unary.expr),
            syn::Expr::Paren(paren) => self.collection_path(&paren.expr),
            _ => None,
        }
    }

    pub(crate) fn count(statement: &syn::Stmt, name: &str, collections: &[String]) -> usize {
        let mut visitor = GuardWrites {
            name,
            collections,
            writes: 0,
            trailing: false,
        };
        visitor.visit_stmt(statement);
        visitor.writes + usize::from(visitor.trailing)
    }
}

impl<'ast> Visit<'ast> for GuardWrites<'_> {
    fn visit_block(&mut self, block: &'ast syn::Block) {
        let before = self.writes;
        for statement in &block.stmts {
            let tail = matches!(statement, syn::Stmt::Expr(syn::Expr::Path(_), None));
            if self.writes > before && !tail {
                self.trailing = true;
            }
            self.visit_stmt(statement);
        }
    }
    fn visit_expr_if(&mut self, expression: &'ast syn::ExprIf) {
        self.visit_expr(&expression.cond);
        let before = self.writes;
        self.visit_block(&expression.then_branch);
        let then_writes = self.writes - before;
        self.writes = before;
        if let Some((_, otherwise)) = &expression.else_branch {
            self.visit_expr(otherwise);
        }
        self.writes = before + then_writes.max(self.writes - before);
    }

    fn visit_expr_match(&mut self, expression: &'ast syn::ExprMatch) {
        self.visit_expr(&expression.expr);
        let before = self.writes;
        let mut largest = 0;
        for arm in &expression.arms {
            self.writes = before;
            self.visit_expr(&arm.body);
            largest = largest.max(self.writes - before);
        }
        self.writes = before + largest;
    }
    fn visit_expr_assign(&mut self, expression: &'ast syn::ExprAssign) {
        self.writes += usize::from(self.rooted(&expression.left));
        syn::visit::visit_expr_assign(self, expression);
    }

    fn visit_expr_binary(&mut self, expression: &'ast syn::ExprBinary) {
        if matches!(
            expression.op,
            syn::BinOp::AddAssign(_)
                | syn::BinOp::SubAssign(_)
                | syn::BinOp::MulAssign(_)
                | syn::BinOp::DivAssign(_)
                | syn::BinOp::RemAssign(_)
                | syn::BinOp::BitXorAssign(_)
                | syn::BinOp::BitAndAssign(_)
                | syn::BinOp::BitOrAssign(_)
                | syn::BinOp::ShlAssign(_)
                | syn::BinOp::ShrAssign(_)
        ) {
            self.writes += usize::from(self.rooted(&expression.left));
        }
        syn::visit::visit_expr_binary(self, expression);
    }

    fn visit_expr_method_call(&mut self, expression: &'ast syn::ExprMethodCall) {
        if self
            .collection_path(&expression.receiver)
            .is_some_and(|path| self.collections.contains(&path))
            && matches!(
                expression.method.to_string().as_str(),
                "insert"
                    | "remove"
                    | "clear"
                    | "push"
                    | "pop"
                    | "extend"
                    | "truncate"
                    | "retain"
                    | "resize"
                    | "append"
                    | "drain"
                    | "sort"
                    | "reverse"
                    | "swap_remove"
            )
        {
            self.writes += 1;
        }
        syn::visit::visit_expr_method_call(self, expression);
    }

    fn visit_expr_closure(&mut self, _: &'ast syn::ExprClosure) {}
    fn visit_expr_async(&mut self, _: &'ast syn::ExprAsync) {}
}
