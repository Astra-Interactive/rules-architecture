use syn::visit::Visit;

pub(crate) struct GuardWrites<'name> {
    name: &'name str,
    collections: &'name [String],
    aliases: &'name [GuardAlias],
    pub(crate) writes: usize,
    pub(crate) trailing: bool,
}

impl GuardWrites<'_> {
    fn find_alias(&self, path: &syn::Path) -> Option<&GuardAlias> {
        self.aliases.iter().find(|alias| path.is_ident(&alias.name))
    }

    fn rooted(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Path(path) => {
                path.path.is_ident(self.name) || self.find_alias(&path.path).is_some()
            }
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
            syn::Expr::Path(path) => self.find_alias(&path.path).map(|alias| alias.path.clone()),
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

    fn exchanges_guard(&self, call: &syn::ExprCall) -> bool {
        let syn::Expr::Path(function) = &*call.func else {
            return false;
        };
        let segments = &function.path.segments;
        let Some(name) = segments.last() else {
            return false;
        };
        let in_mem = segments.len() == 1
            || segments
                .iter()
                .rev()
                .nth(1)
                .is_some_and(|owner| owner.ident == "mem");
        in_mem
            && matches!(name.ident.to_string().as_str(), "take" | "replace" | "swap")
            && matches!(call.args.first(), Some(syn::Expr::Reference(reference))
                if reference.mutability.is_some() && self.rooted(&reference.expr))
    }

    pub(crate) fn collect_aliases(statements: &[syn::Stmt], name: &str) -> Vec<GuardAlias> {
        let mut aliases: Vec<GuardAlias> = Vec::new();
        for statement in statements {
            let syn::Stmt::Local(local) = statement else {
                continue;
            };
            let syn::Pat::Ident(binding) = &local.pat else {
                continue;
            };
            let Some(syn::Expr::Reference(reference)) = local.init.as_ref().map(|init| &*init.expr)
            else {
                continue;
            };
            if reference.mutability.is_none() {
                continue;
            }
            let guard = GuardWrites {
                name,
                collections: &[],
                aliases: &aliases,
                writes: 0,
                trailing: false,
            };
            if let Some(path) = guard.collection_path(&reference.expr) {
                aliases.push(GuardAlias {
                    name: binding.ident.to_string(),
                    path,
                });
            }
        }
        aliases
    }

    pub(crate) fn count(
        statement: &syn::Stmt,
        name: &str,
        collections: &[String],
        aliases: &[GuardAlias],
    ) -> usize {
        let mut visitor = GuardWrites {
            name,
            collections,
            aliases,
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
            let tail = matches!(
                statement,
                syn::Stmt::Expr(syn::Expr::Path(_) | syn::Expr::Lit(_), None)
            );
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
            self.visit_pat(&arm.pat);
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
                    | "push_back"
                    | "push_front"
                    | "pop_back"
                    | "pop_front"
                    | "extend"
                    | "truncate"
                    | "retain"
                    | "resize"
                    | "append"
                    | "drain"
                    | "sort"
                    | "sort_by"
                    | "sort_by_key"
                    | "sort_by_cached_key"
                    | "sort_unstable"
                    | "sort_unstable_by"
                    | "sort_unstable_by_key"
                    | "reverse"
                    | "swap_remove"
                    | "take"
                    | "replace"
                    | "get_or_insert"
                    | "get_or_insert_with"
            )
        {
            self.writes += 1;
        }
        syn::visit::visit_expr_method_call(self, expression);
    }

    fn visit_expr_call(&mut self, expression: &'ast syn::ExprCall) {
        self.writes += usize::from(self.exchanges_guard(expression));
        syn::visit::visit_expr_call(self, expression);
    }

    fn visit_expr_closure(&mut self, _: &'ast syn::ExprClosure) {}
    fn visit_expr_async(&mut self, _: &'ast syn::ExprAsync) {}
}

pub(crate) struct GuardAlias {
    name: String,
    path: String,
}
