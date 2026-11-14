use std::collections::BTreeMap;
use syn::visit::Visit;

pub(crate) struct ScopeOriginVisitor<'name> {
    type_name: &'name str,
    field: &'name str,
    locals: BTreeMap<String, bool>,
    origins: Vec<bool>,
}

impl<'name> ScopeOriginVisitor<'name> {
    pub(crate) fn collect(
        type_name: &'name str,
        field: &'name str,
        block: &syn::Block,
    ) -> Vec<bool> {
        let mut visitor = ScopeOriginVisitor {
            type_name,
            field,
            locals: BTreeMap::new(),
            origins: Vec::new(),
        };
        visitor.visit_block(block);
        visitor.origins
    }

    fn segment_names(path: &syn::Path) -> Vec<String> {
        path.segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect()
    }

    fn is_own_type(&self, path: &syn::Path) -> bool {
        path.segments
            .last()
            .is_some_and(|segment| segment.ident == "Self" || segment.ident == self.type_name)
    }

    fn is_owned(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Paren(parenthesized) => self.is_owned(&parenthesized.expr),
            syn::Expr::Path(path) => path
                .path
                .get_ident()
                .is_some_and(|name| self.locals.get(&name.to_string()).copied().unwrap_or(false)),
            syn::Expr::Call(call) => {
                let syn::Expr::Path(callee) = call.func.as_ref() else {
                    return false;
                };
                let names = Self::segment_names(&callee.path);
                let builds_scope = ["new", "default"]
                    .iter()
                    .any(|name| names.ends_with(&["TaskScope".to_owned(), (*name).to_owned()]));
                let builds_default_arc = names.ends_with(&["Arc".to_owned(), "default".to_owned()]);
                let wraps_owned = names.ends_with(&["Arc".to_owned(), "new".to_owned()])
                    && call.args.len() == 1
                    && call
                        .args
                        .first()
                        .is_some_and(|argument| self.is_owned(argument));
                builds_scope || builds_default_arc || wraps_owned
            }
            _ => false,
        }
    }
}

impl<'ast> Visit<'ast> for ScopeOriginVisitor<'_> {
    fn visit_local(&mut self, local: &'ast syn::Local) {
        syn::visit::visit_local(self, local);
        let name = match &local.pat {
            syn::Pat::Ident(binding) => Some(binding.ident.to_string()),
            syn::Pat::Type(typed) => match typed.pat.as_ref() {
                syn::Pat::Ident(binding) => Some(binding.ident.to_string()),
                _ => None,
            },
            _ => None,
        };
        let Some(name) = name else {
            return;
        };
        let is_owned = local
            .init
            .as_ref()
            .is_some_and(|init| self.is_owned(&init.expr));
        self.locals.insert(name, is_owned);
    }

    fn visit_expr_struct(&mut self, literal: &'ast syn::ExprStruct) {
        if self.is_own_type(&literal.path) {
            for assigned in &literal.fields {
                let key = match &assigned.member {
                    syn::Member::Named(name) => name.to_string(),
                    syn::Member::Unnamed(index) => index.index.to_string(),
                };
                if key == self.field {
                    let is_owned = self.is_owned(&assigned.expr);
                    self.origins.push(is_owned);
                }
            }
        }
        syn::visit::visit_expr_struct(self, literal);
    }

    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if let syn::Expr::Path(callee) = call.func.as_ref()
            && self.is_own_type(&callee.path)
            && let Ok(position) = self.field.parse::<usize>()
            && let Some(argument) = call.args.iter().nth(position)
        {
            let is_owned = self.is_owned(argument);
            self.origins.push(is_owned);
        }
        syn::visit::visit_expr_call(self, call);
    }
}
