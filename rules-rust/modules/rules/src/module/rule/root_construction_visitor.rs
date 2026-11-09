use crate::check::model::RuleContext;
use crate::module::util::resolved_di_type;
use crate::source::util::find_node_span;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct RootConstructionVisitor<'context> {
    context: &'context RuleContext,
    ordinal: usize,
    module: usize,
    invalid: Vec<crate::source::model::NodeSpan>,
}

impl<'context> RootConstructionVisitor<'context> {
    fn associated_factory(&self, path: &syn::Path) -> bool {
        let Some(name) = path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
        else {
            return false;
        };
        let mut owner = path.clone();
        owner.segments.pop();
        owner.segments.pop_punct();
        self.context.resolve_workspace_path(self.ordinal, self.module, &owner).iter().any(|target| {
            let Some(identity) = target.path.item else { return false; };
            if resolved_di_type(self.context, target) { return false; }
            let index = &self.context.indexes[target.index];
            index.implementations.iter().filter(|implementation| crate::module::util::owns_implementation(index, identity, implementation)).any(|implementation| implementation.syntax.items.iter().any(|member| {
                let syn::ImplItem::Fn(method) = member else { return false; };
                if method.sig.ident != name || method.sig.receiver().is_some() { return false; }
                let syn::ReturnType::Type(_, typed) = &method.sig.output else { return false; };
                matches!(typed.as_ref(), syn::Type::Path(path) if path.path.is_ident("Self")) || self.context.resolve_workspace_type(target.index, implementation.module, typed).iter().any(|value| value.index == target.index && value.path.item == Some(identity))
            }))
        })
    }

    fn invalid_call(&self, call: &syn::ExprCall) -> bool {
        let syn::Expr::Path(callee) = call.func.as_ref() else {
            return false;
        };
        let mut path = callee.path.clone();
        let constructor = path.segments.last().is_some_and(|segment| {
            let name = segment.ident.to_string();
            name == "new"
                || name == "default"
                || name.starts_with("with_")
                || name.starts_with("from_")
        });
        let qualified = path.segments.len() >= 2;
        if !qualified || !constructor {
            return (qualified && self.associated_factory(&path))
                || self
                    .context
                    .resolve_workspace_path(self.ordinal, self.module, &path)
                    .iter()
                    .any(|target| {
                        let Some(identity) = target.path.item else {
                            return false;
                        };
                        let item = &self.context.indexes[target.index].items[identity];
                        let syn::Item::Fn(function) = &item.syntax else {
                            return false;
                        };
                        let syn::ReturnType::Type(_, typed) = &function.sig.output else {
                            return false;
                        };
                        self.context
                            .resolve_workspace_type(target.index, item.module, typed)
                            .iter()
                            .any(|value| !resolved_di_type(self.context, value))
                    });
        }
        path.segments.pop();
        path.segments.pop_punct();
        let targets = self
            .context
            .resolve_workspace_path(self.ordinal, self.module, &path);
        targets.is_empty()
            || targets
                .iter()
                .any(|target| !resolved_di_type(self.context, target))
    }
    pub(crate) fn collect(
        context: &'context RuleContext,
        ordinal: usize,
        module: usize,
        block: &syn::Block,
    ) -> Vec<crate::source::model::NodeSpan> {
        let mut visitor = Self {
            context,
            ordinal,
            module,
            invalid: Vec::new(),
        };
        visitor.visit_block(block);
        visitor.invalid
    }
}

impl<'ast> Visit<'ast> for RootConstructionVisitor<'_> {
    fn visit_expr_struct(&mut self, expression: &'ast syn::ExprStruct) {
        if self
            .context
            .resolve_workspace_path(self.ordinal, self.module, &expression.path)
            .iter()
            .any(|target| !resolved_di_type(self.context, target))
        {
            self.invalid.push(find_node_span(expression.span()));
        }
        syn::visit::visit_expr_struct(self, expression);
    }

    fn visit_expr_path(&mut self, expression: &'ast syn::ExprPath) {
        if self.context.resolve_workspace_path(self.ordinal, self.module, &expression.path).iter().any(|target| target.path.item.is_some_and(|identity| matches!(&self.context.indexes[target.index].items[identity].syntax, syn::Item::Struct(structure) if matches!(structure.fields, syn::Fields::Unit))) && !resolved_di_type(self.context, target)) {
            self.invalid.push(find_node_span(expression.span()));
        }
    }

    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if self.invalid_call(call) {
            self.invalid.push(find_node_span(call.span()));
        }
        for argument in &call.args {
            self.visit_expr(argument);
        }
    }
}
