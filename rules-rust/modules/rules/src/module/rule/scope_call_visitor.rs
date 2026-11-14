use crate::source::util::parse_macro_arguments;
use std::collections::BTreeMap;
use syn::visit::Visit;

pub(crate) struct ScopeCall {
    pub(crate) field: String,
    pub(crate) method: String,
}

pub(crate) struct ScopeCallVisitor {
    calls: Vec<ScopeCall>,
    aliases: BTreeMap<String, String>,
}

impl ScopeCallVisitor {
    pub(crate) fn collect(block: &syn::Block) -> Vec<ScopeCall> {
        let mut visitor = ScopeCallVisitor {
            calls: Vec::new(),
            aliases: BTreeMap::new(),
        };
        visitor.visit_block(block);
        visitor.calls
    }

    fn member_key(member: &syn::Member) -> String {
        match member {
            syn::Member::Named(name) => name.to_string(),
            syn::Member::Unnamed(index) => index.index.to_string(),
        }
    }

    fn is_arc_clone(callee: &syn::ExprPath) -> bool {
        let names: Vec<String> = callee
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect();
        names.ends_with(&["Arc".to_owned(), "clone".to_owned()])
    }

    fn receiver_field(&self, expression: &syn::Expr) -> Option<String> {
        match expression {
            syn::Expr::Reference(reference) => self.receiver_field(&reference.expr),
            syn::Expr::Paren(parenthesized) => self.receiver_field(&parenthesized.expr),
            syn::Expr::Field(field) => {
                let is_self = matches!(field.base.as_ref(), syn::Expr::Path(path) if path.path.is_ident("self"));
                is_self.then(|| Self::member_key(&field.member))
            }
            syn::Expr::Path(path) => path
                .path
                .get_ident()
                .and_then(|name| self.aliases.get(&name.to_string()).cloned()),
            syn::Expr::MethodCall(call)
                if call.args.is_empty()
                    && ["as_ref", "clone", "deref"].contains(&call.method.to_string().as_str()) =>
            {
                self.receiver_field(&call.receiver)
            }
            syn::Expr::Call(call) if call.args.len() == 1 => {
                let syn::Expr::Path(callee) = call.func.as_ref() else {
                    return None;
                };
                if !Self::is_arc_clone(callee) {
                    return None;
                }
                call.args
                    .first()
                    .and_then(|argument| self.receiver_field(argument))
            }
            _ => None,
        }
    }
}

impl<'ast> Visit<'ast> for ScopeCallVisitor {
    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        for argument in &parse_macro_arguments(item) {
            self.visit_expr(argument);
        }
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        let outer = self.aliases.clone();
        syn::visit::visit_block(self, block);
        self.aliases = outer;
    }

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
        let field = local
            .init
            .as_ref()
            .and_then(|init| self.receiver_field(&init.expr));
        let Some(name) = name else {
            return;
        };
        match field {
            Some(field) => {
                self.aliases.insert(name, field);
            }
            None => {
                self.aliases.remove(&name);
            }
        }
    }

    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if let syn::Expr::Path(callee) = call.func.as_ref()
            && callee.path.segments.len() > 1
            && let Some(method) = callee.path.segments.last()
            && let Some(field) = call
                .args
                .first()
                .and_then(|argument| self.receiver_field(argument))
        {
            self.calls.push(ScopeCall {
                field,
                method: method.ident.to_string(),
            });
        }
        syn::visit::visit_expr_call(self, call);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        if let Some(field) = self.receiver_field(&call.receiver) {
            self.calls.push(ScopeCall {
                field,
                method: call.method.to_string(),
            });
        }
        syn::visit::visit_expr_method_call(self, call);
    }
}
