use syn::visit::Visit;

pub(crate) struct ClosureEffects {
    bound: Vec<String>,
    closure_depth: usize,
    nested: bool,
    pub(crate) assigns_capture: bool,
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
            syn::Pat::Ident(name) => self.bound.push(name.ident.to_string()),
            syn::Pat::Type(typed) => self.bind(&typed.pat),
            syn::Pat::Tuple(tuple) => {
                for element in &tuple.elems {
                    self.bind(element);
                }
            }
            _ => {}
        }
    }

    fn assignment(&mut self, expression: &syn::Expr) {
        match expression {
            syn::Expr::Path(path) if path.path.segments.len() == 1 => {
                if !self.bound.iter().any(|name| path.path.is_ident(name)) {
                    self.assigns_capture = true;
                }
            }
            syn::Expr::Field(field) => self.assignment(&field.base),
            syn::Expr::Index(index) => self.assignment(&index.expr),
            syn::Expr::Unary(unary) => self.assignment(&unary.expr),
            syn::Expr::Paren(paren) => self.assignment(&paren.expr),
            _ => {}
        }
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
        if matches!(&*expression.receiver, syn::Expr::Path(path) if !self.bound.iter().any(|name| path.path.is_ident(name)))
        {
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
