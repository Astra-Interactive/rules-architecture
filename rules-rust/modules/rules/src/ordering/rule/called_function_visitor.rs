use crate::source::util::parse_macro_arguments;
use std::collections::{HashMap, HashSet};
use syn::visit::Visit;

pub(crate) struct CalledFunctionVisitor<'scope> {
    own_name: Option<&'scope str>,
    calls: HashSet<String>,
    bindings: Vec<HashMap<String, Option<String>>>,
    own_receivers: HashSet<String>,
}

impl<'scope> CalledFunctionVisitor<'scope> {
    pub(crate) fn new(own_name: Option<&'scope str>) -> Self {
        Self {
            own_name,
            calls: HashSet::new(),
            bindings: vec![HashMap::new()],
            own_receivers: HashSet::new(),
        }
    }

    fn is_own_type(&self, ty: &syn::Type) -> bool {
        match ty {
            syn::Type::Reference(reference) => self.is_own_type(&reference.elem),
            syn::Type::Paren(parenthesized) => self.is_own_type(&parenthesized.elem),
            syn::Type::Path(value) => {
                value.qself.is_none()
                    && (value.path.is_ident("Self")
                        || self.own_name.is_some_and(|name| value.path.is_ident(name)))
            }
            _ => false,
        }
    }

    fn register_own_receivers(&mut self, signature: &syn::Signature) {
        if self.own_name.is_none() {
            return;
        }
        for input in &signature.inputs {
            if let syn::FnArg::Typed(parameter) = input
                && let syn::Pat::Ident(binding) = parameter.pat.as_ref()
                && self.is_own_type(&parameter.ty)
            {
                self.own_receivers.insert(binding.ident.to_string());
            }
        }
    }

    fn record_path_arguments(
        &mut self,
        arguments: &syn::punctuated::Punctuated<syn::Expr, syn::token::Comma>,
    ) {
        for argument in arguments {
            if let syn::Expr::Path(path) = argument
                && let Some(name) = self.called_name(path)
            {
                self.calls.insert(name);
            }
        }
    }

    fn import_binding(&mut self, tree: &syn::UseTree, prefix: &[String]) {
        match tree {
            syn::UseTree::Path(path) => {
                let mut prefix = prefix.to_vec();
                prefix.push(path.ident.to_string());
                self.import_binding(&path.tree, &prefix);
            }
            syn::UseTree::Group(group) => {
                for tree in &group.items {
                    self.import_binding(tree, prefix);
                }
            }
            syn::UseTree::Name(name) => {
                let target =
                    (self.own_name.is_none() && prefix == ["self"]).then(|| name.ident.to_string());
                if let Some(scope) = self.bindings.last_mut() {
                    scope.insert(name.ident.to_string(), target);
                }
            }
            syn::UseTree::Rename(rename) => {
                let target = (self.own_name.is_none() && prefix == ["self"])
                    .then(|| rename.ident.to_string());
                if let Some(scope) = self.bindings.last_mut() {
                    scope.insert(rename.rename.to_string(), target);
                }
            }
            syn::UseTree::Glob(_) => {}
        }
    }

    pub(crate) fn with_imports(mut self, items: &[syn::Item]) -> Self {
        for item in items {
            if let syn::Item::Use(import) = item {
                self.import_binding(&import.tree, &[]);
            }
        }
        self
    }

    fn merge_bindings(&mut self, alternative: &[HashMap<String, Option<String>>]) {
        for (scope, other) in self.bindings.iter_mut().zip(alternative) {
            for (name, target) in scope {
                if other.get(name) != Some(target) {
                    *target = None;
                }
            }
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

    fn called_name(&self, path: &syn::ExprPath) -> Option<String> {
        let segments = &path.path.segments;
        if let Some(name) = path.path.get_ident()
            && let Some(binding) = self
                .bindings
                .iter()
                .rev()
                .find_map(|scope| scope.get(&name.to_string()))
        {
            return binding.clone();
        }
        let local_path =
            segments.len() == 1 || (segments.len() == 2 && segments[0].ident == "self");
        let free = self.own_name.is_none() && path.qself.is_none() && local_path;
        let own = self
            .own_name
            .is_some_and(|name| Self::calls_owner(path, name));
        segments
            .last()
            .filter(|_| free || own)
            .map(|segment| segment.ident.to_string())
    }

    pub(crate) fn collect_calls(
        mut self,
        signature: &syn::Signature,
        block: &syn::Block,
    ) -> HashSet<String> {
        for input in &signature.inputs {
            if let syn::FnArg::Typed(parameter) = input {
                self.visit_pat(&parameter.pat);
            }
        }
        self.register_own_receivers(signature);
        self.visit_block(block);
        self.calls
    }
}

impl<'ast> Visit<'ast> for CalledFunctionVisitor<'_> {
    fn visit_block(&mut self, block: &'ast syn::Block) {
        self.bindings.push(HashMap::new());
        for statement in &block.stmts {
            if let syn::Stmt::Item(syn::Item::Use(import)) = statement {
                self.import_binding(&import.tree, &[]);
            }
            if let syn::Stmt::Item(syn::Item::Fn(function)) = statement
                && let Some(scope) = self.bindings.last_mut()
            {
                scope.insert(function.sig.ident.to_string(), None);
            }
        }
        syn::visit::visit_block(self, block);
        self.bindings.pop();
    }

    fn visit_pat_ident(&mut self, binding: &'ast syn::PatIdent) {
        self.own_receivers.remove(&binding.ident.to_string());
        if let Some(scope) = self.bindings.last_mut() {
            scope.insert(binding.ident.to_string(), None);
        }
        syn::visit::visit_pat_ident(self, binding);
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        let target = local.init.as_ref().and_then(|initializer| {
            let syn::Expr::Path(path) = initializer.expr.as_ref() else {
                return None;
            };
            self.called_name(path)
        });
        for attribute in &local.attrs {
            self.visit_attribute(attribute);
        }
        if let Some(initializer) = &local.init {
            self.visit_expr(&initializer.expr);
            if let Some((_, diverging)) = &initializer.diverge {
                self.visit_expr(diverging);
            }
        }
        self.visit_pat(&local.pat);
        let binding = match &local.pat {
            syn::Pat::Ident(binding) => Some(binding),
            syn::Pat::Type(typed) => match typed.pat.as_ref() {
                syn::Pat::Ident(binding) => Some(binding),
                _ => None,
            },
            _ => None,
        };
        if let Some(binding) = binding
            && let Some(scope) = self.bindings.last_mut()
        {
            scope.insert(binding.ident.to_string(), target);
        }
    }

    fn visit_expr_assign(&mut self, assignment: &'ast syn::ExprAssign) {
        syn::visit::visit_expr_assign(self, assignment);
        if let syn::Expr::Path(path) = assignment.left.as_ref()
            && let Some(name) = path.path.get_ident()
            && let Some(scope) = self
                .bindings
                .iter_mut()
                .rev()
                .find(|scope| scope.contains_key(&name.to_string()))
        {
            scope.insert(name.to_string(), None);
        }
    }

    fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure) {
        self.bindings.push(HashMap::new());
        syn::visit::visit_expr_closure(self, closure);
        self.bindings.pop();
    }

    fn visit_expr_for_loop(&mut self, expression: &'ast syn::ExprForLoop) {
        self.visit_expr(&expression.expr);
        self.bindings.push(HashMap::new());
        self.visit_pat(&expression.pat);
        self.visit_block(&expression.body);
        self.bindings.pop();
    }

    fn visit_expr_let(&mut self, expression: &'ast syn::ExprLet) {
        self.visit_expr(&expression.expr);
        self.visit_pat(&expression.pat);
    }

    fn visit_expr_if(&mut self, expression: &'ast syn::ExprIf) {
        self.bindings.push(HashMap::new());
        self.visit_expr(&expression.cond);
        let mut condition = self.bindings.clone();
        condition.pop();
        self.visit_block(&expression.then_branch);
        self.bindings.pop();
        let then_bindings = std::mem::replace(&mut self.bindings, condition);
        if let Some((_, branch)) = &expression.else_branch {
            self.visit_expr(branch);
        }
        self.merge_bindings(&then_bindings);
    }

    fn visit_expr_while(&mut self, expression: &'ast syn::ExprWhile) {
        self.bindings.push(HashMap::new());
        self.visit_expr(&expression.cond);
        self.visit_block(&expression.body);
        self.bindings.pop();
    }

    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        self.bindings.push(HashMap::new());
        syn::visit::visit_arm(self, arm);
        self.bindings.pop();
    }

    fn visit_expr_match(&mut self, expression: &'ast syn::ExprMatch) {
        self.visit_expr(&expression.expr);
        let before = self.bindings.clone();
        let mut outcomes = Vec::new();
        for arm in &expression.arms {
            self.bindings = before.clone();
            self.visit_arm(arm);
            outcomes.push(self.bindings.clone());
        }
        self.bindings = before;
        for outcome in outcomes {
            self.merge_bindings(&outcome);
        }
    }

    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        self.bindings.push(HashMap::new());
        for input in &function.sig.inputs {
            if let syn::FnArg::Typed(parameter) = input {
                self.visit_pat(&parameter.pat);
            }
        }
        self.visit_block(&function.block);
        self.bindings.pop();
    }

    fn visit_expr_call(&mut self, expression: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = &*expression.func
            && let Some(name) = self.called_name(path)
        {
            self.calls.insert(name);
        }
        self.record_path_arguments(&expression.args);
        syn::visit::visit_expr_call(self, expression);
    }

    fn visit_expr_method_call(&mut self, expression: &'ast syn::ExprMethodCall) {
        let has_owner = self.own_name.is_some();
        let has_own_receiver = matches!(
            &*expression.receiver,
            syn::Expr::Path(path)
                if path.path.is_ident("self")
                    || path.path.get_ident().is_some_and(|name| {
                        self.own_receivers.contains(&name.to_string())
                    })
        );
        if has_owner && has_own_receiver {
            self.calls.insert(expression.method.to_string());
        }
        self.record_path_arguments(&expression.args);
        syn::visit::visit_expr_method_call(self, expression);
    }

    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        for argument in &parse_macro_arguments(item) {
            self.visit_expr(argument);
        }
    }
}
