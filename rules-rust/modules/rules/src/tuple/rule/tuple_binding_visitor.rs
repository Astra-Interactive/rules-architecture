use crate::check::model::RuleContext;
use crate::source::model::{NodeSpan, Resolution};
use crate::source::util::find_node_span;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct TupleBindingVisitor<'context> {
    context: &'context RuleContext,
    ordinal: usize,
    module: usize,
    spans: Vec<NodeSpan>,
}

impl<'context> TupleBindingVisitor<'context> {
    pub(crate) fn new(context: &'context RuleContext, ordinal: usize, module: usize) -> Self {
        Self {
            context,
            ordinal,
            module,
            spans: Vec::new(),
        }
    }
    pub(crate) fn collect_item(mut self, item: &syn::Item) -> Vec<NodeSpan> {
        self.visit_item(item);
        self.spans
    }
    pub(crate) fn collect_impl(mut self, implementation: &syn::ItemImpl) -> Vec<NodeSpan> {
        self.visit_item_impl(implementation);
        self.spans
    }

    fn tuple_return(&self, ordinal: usize, module: usize, output: &syn::ReturnType) -> bool {
        let syn::ReturnType::Type(_, typed) = output else {
            return false;
        };
        self.context.resolve_type_syntax(ordinal, module, typed).iter().any(|resolved| matches!(&resolved.typed, syn::Type::Tuple(tuple) if tuple.elems.len() >= 2))
    }

    fn call_tuple(&self, call: &syn::ExprCall) -> bool {
        let syn::Expr::Path(callee) = call.func.as_ref() else {
            return false;
        };
        let Some(name) = callee.path.segments.last() else {
            return false;
        };
        if ![
            "split_at",
            "split_at_mut",
            "partition",
            "unzip",
            "channel",
            "unbounded_channel",
        ]
        .iter()
        .any(|required| name.ident == *required)
        {
            return false;
        }
        let targets = self
            .context
            .resolve_workspace_path(self.ordinal, self.module, &callee.path);
        if !targets.is_empty() {
            return targets.iter().any(|target| target.path.item.is_some_and(|identity| {
                let item = &self.context.indexes[target.index].items[identity];
                matches!(&item.syntax, syn::Item::Fn(function) if self.tuple_return(target.index, item.module, &function.sig.output))
            }));
        }
        let Resolution::External(path) =
            self.context.indexes[self.ordinal].resolve_path(self.module, &callee.path)
        else {
            return false;
        };
        let path = self.context.resolve_external_identity(self.ordinal, &path);
        (["tokio", "futures", "futures_channel"].contains(&path.package.as_str())
            && path.segments.iter().any(|segment| {
                ["mpsc", "oneshot", "watch", "broadcast"].contains(&segment.as_str())
            }))
            || (path.package == "std" && path.segments.iter().any(|segment| segment == "mpsc"))
    }

    fn tuple_expression(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Tuple(tuple) => tuple.elems.len() >= 2,
            syn::Expr::Paren(paren) => self.tuple_expression(&paren.expr),
            syn::Expr::Group(group) => self.tuple_expression(&group.expr),
            syn::Expr::Call(call) => self.call_tuple(call),
            syn::Expr::MethodCall(call) => {
                ["split_at", "split_at_mut"]
                    .iter()
                    .any(|name| call.method == *name)
                    && matches!(
                        call.receiver.as_ref(),
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(_),
                            ..
                        })
                    )
            }
            _ => false,
        }
    }
}

impl<'ast> Visit<'ast> for TupleBindingVisitor<'_> {
    fn visit_local(&mut self, local: &'ast syn::Local) {
        let named = match &local.pat {
            syn::Pat::Ident(_) => true,
            syn::Pat::Type(typed) => matches!(typed.pat.as_ref(), syn::Pat::Ident(_)),
            _ => false,
        };
        if named
            && local
                .init
                .as_ref()
                .is_some_and(|initializer| self.tuple_expression(&initializer.expr))
        {
            self.spans.push(find_node_span(local.span()));
        }
        syn::visit::visit_local(self, local);
    }
}
