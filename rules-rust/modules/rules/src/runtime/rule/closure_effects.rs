use syn::visit::Visit;

pub(crate) struct ClosureEffects {
    bound: Vec<String>,
    closure_depth: usize,
    nested: bool,
    pub(crate) assigns_capture: bool,
    pub(crate) borrows_capture_mutably: bool,
    pub(crate) methods: Vec<syn::ExprMethodCall>,
    pub(crate) calls: Vec<syn::ExprCall>,
    pub(crate) inline_logic: bool,
    pub(crate) captured_methods: Vec<syn::ExprMethodCall>,
    pub(crate) macros: Vec<syn::Macro>,
}

impl ClosureEffects {
    pub(crate) fn collect(expression: &syn::Expr) -> Self {
        Self::collect_scope(expression, false)
    }

    pub(crate) fn collect_nested(expression: &syn::Expr) -> Self {
        Self::collect_scope(expression, true)
    }

    fn collect_scope(expression: &syn::Expr, nested: bool) -> Self {
        let mut visitor = Self {
            bound: Vec::new(),
            closure_depth: 0,
            nested,
            assigns_capture: false,
            borrows_capture_mutably: false,
            methods: Vec::new(),
            calls: Vec::new(),
            inline_logic: false,
            captured_methods: Vec::new(),
            macros: Vec::new(),
        };
        if let syn::Expr::Async(future) = expression {
            visitor.visit_block(&future.block);
        } else {
            visitor.visit_expr(expression);
        }
        visitor
    }

    fn bind(&mut self, pattern: &syn::Pat) {
        match pattern {
            syn::Pat::Ident(name) => {
                self.bound.push(name.ident.to_string());
                if let Some((_, subpattern)) = &name.subpat {
                    self.bind(subpattern);
                }
            }
            syn::Pat::Type(typed) => self.bind(&typed.pat),
            syn::Pat::Paren(paren) => self.bind(&paren.pat),
            syn::Pat::Guard(guarded) => self.bind(&guarded.pat),
            syn::Pat::Reference(reference) => self.bind(&reference.pat),
            syn::Pat::Tuple(tuple) => self.bind_all(&tuple.elems),
            syn::Pat::TupleStruct(tuple) => self.bind_all(&tuple.elems),
            syn::Pat::Slice(slice) => self.bind_all(&slice.elems),
            syn::Pat::Or(alternatives) => self.bind_all(&alternatives.cases),
            syn::Pat::Struct(structure) => {
                for field in &structure.fields {
                    self.bind(&field.pat);
                }
            }
            _ => {}
        }
    }

    fn bind_all<'pattern>(&mut self, patterns: impl IntoIterator<Item = &'pattern syn::Pat>) {
        for pattern in patterns {
            self.bind(pattern);
        }
    }

    fn is_capture_place(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Path(path) => {
                path.path.segments.len() == 1
                    && !self.bound.iter().any(|name| path.path.is_ident(name))
            }
            syn::Expr::Field(field) => self.is_capture_place(&field.base),
            syn::Expr::Index(index) => self.is_capture_place(&index.expr),
            syn::Expr::Unary(unary) => self.is_capture_place(&unary.expr),
            syn::Expr::Paren(paren) => self.is_capture_place(&paren.expr),
            _ => false,
        }
    }

    fn assignment(&mut self, expression: &syn::Expr) {
        self.assigns_capture |= self.is_capture_place(expression);
    }
}

impl<'ast> Visit<'ast> for ClosureEffects {
    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        self.macros.push(item.clone());
        if !self.nested {
            return;
        }
        let Ok(arguments) = syn::parse::Parser::parse2(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
            item.tokens.clone(),
        ) else {
            return;
        };
        for argument in &arguments {
            self.visit_expr(argument);
        }
    }
    fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure) {
        if self.closure_depth > 0 && !self.nested {
            return;
        }
        self.closure_depth += 1;
        let count = self.bound.len();
        for parameter in &closure.inputs {
            self.bind(parameter);
        }
        self.visit_expr(&closure.body);
        self.bound.truncate(count);
        self.closure_depth -= 1;
    }

    fn visit_expr_async(&mut self, _: &'ast syn::ExprAsync) {}
    fn visit_item_fn(&mut self, _: &'ast syn::ItemFn) {}

    fn visit_block(&mut self, block: &'ast syn::Block) {
        let count = self.bound.len();
        syn::visit::visit_block(self, block);
        self.bound.truncate(count);
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let Some(init) = &local.init {
            self.visit_expr(&init.expr);
        }
        self.bind(&local.pat);
    }

    fn visit_expr_let(&mut self, expression: &'ast syn::ExprLet) {
        self.visit_expr(&expression.expr);
        self.bind(&expression.pat);
    }

    fn visit_expr_if(&mut self, expression: &'ast syn::ExprIf) {
        let count = self.bound.len();
        self.visit_expr(&expression.cond);
        self.visit_block(&expression.then_branch);
        self.bound.truncate(count);
        if let Some((_, otherwise)) = &expression.else_branch {
            self.visit_expr(otherwise);
        }
    }

    fn visit_expr_while(&mut self, expression: &'ast syn::ExprWhile) {
        let count = self.bound.len();
        self.visit_expr(&expression.cond);
        self.visit_block(&expression.body);
        self.bound.truncate(count);
    }

    fn visit_expr_for_loop(&mut self, expression: &'ast syn::ExprForLoop) {
        self.visit_expr(&expression.expr);
        let count = self.bound.len();
        self.bind(&expression.pat);
        self.visit_block(&expression.body);
        self.bound.truncate(count);
    }

    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        let count = self.bound.len();
        self.bind(&arm.pat);
        if let syn::Pat::Guard(guarded) = &arm.pat {
            self.visit_expr(&guarded.guard);
        }
        self.visit_expr(&arm.body);
        self.bound.truncate(count);
    }

    fn visit_expr_reference(&mut self, expression: &'ast syn::ExprReference) {
        if expression.mutability.is_some() && self.is_capture_place(&expression.expr) {
            self.borrows_capture_mutably = true;
        }
        syn::visit::visit_expr_reference(self, expression);
    }

    fn visit_expr_assign(&mut self, expression: &'ast syn::ExprAssign) {
        self.inline_logic = true;
        self.assignment(&expression.left);
        syn::visit::visit_expr_assign(self, expression);
    }

    fn visit_expr_binary(&mut self, expression: &'ast syn::ExprBinary) {
        self.inline_logic = true;
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
            self.assignment(&expression.left);
        }
        syn::visit::visit_expr_binary(self, expression);
    }

    fn visit_expr_method_call(&mut self, expression: &'ast syn::ExprMethodCall) {
        if self.is_capture_place(&expression.receiver) {
            self.captured_methods.push(expression.clone());
        }
        self.methods.push(expression.clone());
        syn::visit::visit_expr_method_call(self, expression);
    }

    fn visit_expr_call(&mut self, expression: &'ast syn::ExprCall) {
        self.calls.push(expression.clone());
        syn::visit::visit_expr_call(self, expression);
    }
}
