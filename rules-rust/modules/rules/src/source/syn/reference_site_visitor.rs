use crate::source::model::{NodeSpan, ReferenceSite, WorkspaceUseKind};
use crate::source::util::find_node_span;
use crate::source::util::parse_macro_arguments;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};

pub(crate) struct ReferenceSiteVisitor {
    pub(crate) sites: Vec<ReferenceSite>,
    kind: WorkspaceUseKind,
    function: Option<String>,
    block: Option<NodeSpan>,
}

impl ReferenceSiteVisitor {
    pub(crate) fn new() -> Self {
        Self {
            sites: Vec::new(),
            kind: WorkspaceUseKind::Value,
            function: None,
            block: None,
        }
    }
}

impl<'ast> Visit<'ast> for ReferenceSiteVisitor {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        self.sites.push(ReferenceSite {
            syntax: path.clone(),
            span: find_node_span(path.span()),
            kind: self.kind,
            function: self.function.clone(),
            block: self.block,
            path: path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect(),
        });
        visit::visit_path(self, path);
    }

    fn visit_type_path(&mut self, path: &'ast syn::TypePath) {
        let previous = self.kind;
        self.kind = WorkspaceUseKind::Type;
        visit::visit_type_path(self, path);
        self.kind = previous;
    }

    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        for argument in &parse_macro_arguments(item) {
            self.visit_expr(argument);
        }
    }

    fn visit_expr_call(&mut self, expression: &'ast syn::ExprCall) {
        let previous = self.kind;
        self.kind = WorkspaceUseKind::Call;
        self.visit_expr(&expression.func);
        self.kind = previous;
        for argument in &expression.args {
            self.visit_expr(argument);
        }
    }

    fn visit_expr_struct(&mut self, expression: &'ast syn::ExprStruct) {
        let previous = self.kind;
        self.kind = WorkspaceUseKind::Construction;
        self.visit_path(&expression.path);
        self.kind = previous;
        for field in &expression.fields {
            self.visit_expr(&field.expr);
        }
        if let Some(rest) = &expression.rest {
            self.visit_expr(rest);
        }
    }

    fn visit_signature(&mut self, signature: &'ast syn::Signature) {
        self.function = Some(signature.ident.to_string());
        visit::visit_signature(self, signature);
    }

    fn visit_item_impl(&mut self, implementation: &'ast syn::ItemImpl) {
        if let Some((path, _)) = &implementation.trait_ {
            let previous = self.kind;
            self.kind = WorkspaceUseKind::TraitImplementation;
            self.visit_path(path);
            self.kind = previous;
        }
        self.visit_type(&implementation.self_ty);
        self.visit_generics(&implementation.generics);
        for item in &implementation.items {
            self.visit_impl_item(item);
        }
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        let previous = self.block;
        self.block = Some(find_node_span(block.span()));
        visit::visit_block(self, block);
        self.block = previous;
    }
}
