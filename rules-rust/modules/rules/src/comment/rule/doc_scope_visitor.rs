use crate::comment::rule::PanicDocVisitor;
use crate::comment::util::{doc_heading, joined_doc, strip_doc_code};
use crate::source::model::NodeSpan;
use syn::visit::Visit;

pub(crate) struct DocScopeVisitor<'context> {
    context: &'context crate::check::model::RuleContext,
    ordinal: usize,
    file: String,
    cfg: crate::source::model::CfgPredicate,
    published: bool,
    trait_impl: bool,
    spans: Vec<NodeSpan>,
}

impl DocScopeVisitor<'_> {
    pub(crate) fn collect(
        context: &crate::check::model::RuleContext,
        ordinal: usize,
        file: &crate::source::model::SourceFile,
        published: bool,
    ) -> Vec<NodeSpan> {
        let mut visitor = DocScopeVisitor {
            context,
            ordinal,
            file: file.path.clone(),
            cfg: crate::source::model::CfgPredicate::All(Vec::new()),
            published,
            trait_impl: false,
            spans: Vec::new(),
        };
        visitor.visit_file(&file.syntax);
        visitor.spans
    }

    fn inspect(&mut self, attributes: &[syn::Attribute], forbidden: bool, panic: bool) {
        if !self
            .cfg
            .and(&crate::source::model::CfgPredicate::from_attributes(
                attributes,
            ))
            .is_satisfiable()
        {
            return;
        }
        let Some((span, text)) = joined_doc(attributes) else {
            return;
        };
        let cleaned = strip_doc_code(&text);
        let labels = cleaned.lines().any(|line| {
            ["Errors:", "Panics:", "Safety:"]
                .iter()
                .any(|label| line.trim_start().starts_with(label))
        });
        let link = text.contains("](")
            || text.contains("[`")
            || text.contains("https://")
            || text.contains("http://");
        let application = !self.published
            && !doc_heading(&cleaned, "Errors")
            && !doc_heading(&cleaned, "Safety")
            && !link;
        let panic = panic && !doc_heading(&cleaned, "Panics");
        if forbidden || labels || application || panic {
            self.spans.push(span);
        }
    }

    fn module_for(&self, span: NodeSpan) -> usize {
        let index = &self.context.indexes[self.ordinal];
        index
            .items
            .iter()
            .find(|item| item.file == self.file && item.span.contains(&span))
            .map(|item| item.module)
            .or_else(|| {
                index
                    .implementations
                    .iter()
                    .find(|item| item.file == self.file && item.span.contains(&span))
                    .map(|item| item.module)
            })
            .unwrap_or(0)
    }

    fn public_function(
        &mut self,
        attributes: &[syn::Attribute],
        visibility: &syn::Visibility,
        block: &syn::Block,
        fallback: NodeSpan,
        signature: &syn::Signature,
    ) {
        let module = self.module_for(fallback);
        let cfg = self
            .cfg
            .and(&crate::source::model::CfgPredicate::from_attributes(
                attributes,
            ));
        if !cfg.is_satisfiable() {
            return;
        }
        let panic = self.published
            && matches!(visibility, syn::Visibility::Public(_))
            && PanicDocVisitor::contains_panic(
                self.context,
                self.ordinal,
                module,
                signature,
                &cfg,
                block,
            );
        if joined_doc(attributes).is_some() {
            self.inspect(attributes, self.trait_impl, panic);
        } else if panic {
            self.spans.push(fallback);
        }
    }
}

impl<'ast> Visit<'ast> for DocScopeVisitor<'_> {
    fn visit_file(&mut self, syntax: &'ast syn::File) {
        self.inspect(&syntax.attrs, false, false);
        syn::visit::visit_file(self, syntax);
    }
    fn visit_item(&mut self, item: &'ast syn::Item) {
        let cfg = self.cfg.clone();
        self.cfg = cfg.and(&crate::source::model::CfgPredicate::from_attributes(
            crate::comment::util::item_doc_attributes(item),
        ));
        if !self.cfg.is_satisfiable() {
            self.cfg = cfg;
            return;
        }
        if !matches!(item, syn::Item::Fn(_)) {
            self.inspect(
                crate::comment::util::item_doc_attributes(item),
                false,
                false,
            );
        }
        syn::visit::visit_item(self, item);
        self.cfg = cfg;
    }
    fn visit_item_impl(&mut self, implementation: &'ast syn::ItemImpl) {
        let previous = self.trait_impl;
        self.trait_impl = implementation.trait_.is_some();
        syn::visit::visit_item_impl(self, implementation);
        self.trait_impl = previous;
    }

    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        let previous = self.trait_impl;
        self.trait_impl = false;
        self.public_function(
            &function.attrs,
            &function.vis,
            &function.block,
            crate::source::util::find_node_span(syn::spanned::Spanned::span(function)),
            &function.sig,
        );
        syn::visit::visit_item_fn(self, function);
        self.trait_impl = previous;
    }

    fn visit_impl_item_fn(&mut self, function: &'ast syn::ImplItemFn) {
        self.public_function(
            &function.attrs,
            &function.vis,
            &function.block,
            crate::source::util::find_node_span(syn::spanned::Spanned::span(function)),
            &function.sig,
        );
        syn::visit::visit_impl_item_fn(self, function);
    }
    fn visit_trait_item_fn(&mut self, function: &'ast syn::TraitItemFn) {
        self.inspect(&function.attrs, false, false);
        syn::visit::visit_trait_item_fn(self, function);
    }
    fn visit_field(&mut self, item: &'ast syn::Field) {
        self.inspect(&item.attrs, false, false);
        syn::visit::visit_field(self, item);
    }
    fn visit_variant(&mut self, item: &'ast syn::Variant) {
        self.inspect(&item.attrs, false, false);
        syn::visit::visit_variant(self, item);
    }
    fn visit_trait_item_type(&mut self, item: &'ast syn::TraitItemType) {
        self.inspect(&item.attrs, false, false);
        syn::visit::visit_trait_item_type(self, item);
    }
    fn visit_trait_item_const(&mut self, item: &'ast syn::TraitItemConst) {
        self.inspect(&item.attrs, false, false);
        syn::visit::visit_trait_item_const(self, item);
    }
    fn visit_impl_item_type(&mut self, item: &'ast syn::ImplItemType) {
        self.inspect(&item.attrs, false, false);
        syn::visit::visit_impl_item_type(self, item);
    }
    fn visit_impl_item_const(&mut self, item: &'ast syn::ImplItemConst) {
        self.inspect(&item.attrs, false, false);
        syn::visit::visit_impl_item_const(self, item);
    }
}
