use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::comment::util::{
    doc_attribute_lines, doc_inline_candidates, doc_resolves, item_doc_attributes, joined_doc,
};
use crate::source::model::{CfgPredicate, SourceFile};
use crate::source::util::find_node_span;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct DocLinkVisitor<'context> {
    context: &'context RuleContext,
    ordinal: usize,
    module: usize,
    owner: Option<(usize, usize)>,
    cfg: CfgPredicate,
    file: String,
    findings: Vec<RuleFinding>,
}

impl DocLinkVisitor<'_> {
    pub(crate) fn collect(
        context: &RuleContext,
        ordinal: usize,
        file: &SourceFile,
    ) -> Vec<RuleFinding> {
        let module = context.indexes[ordinal]
            .modules
            .iter()
            .find(|module| module.module == file.module && module.file == file.path)
            .map_or(0, |module| module.id);
        let cfg = context.indexes[ordinal].modules[module].cfg.clone();
        let mut visitor = DocLinkVisitor {
            context,
            ordinal,
            module,
            owner: None,
            cfg,
            file: file.path.clone(),
            findings: Vec::new(),
        };
        visitor.visit_file(&file.syntax);
        visitor.findings
    }

    fn inspect(&mut self, attributes: &[syn::Attribute]) {
        let cfg = self.cfg.and(&CfgPredicate::from_attributes(attributes));
        if !cfg.is_satisfiable() {
            return;
        }
        let Some((fallback, text)) = joined_doc(attributes) else {
            return;
        };
        let lines = doc_attribute_lines(attributes);
        for (line, candidate) in doc_inline_candidates(&text) {
            if doc_resolves(
                self.context,
                self.ordinal,
                self.module,
                self.owner,
                &candidate,
                &cfg,
            ) {
                self.findings.push(build_node_finding(
                    "IntraDocLink",
                    &self.file,
                    lines.get(line).copied().unwrap_or(fallback),
                    candidate,
                    "link an item that resolves in this scope with an intra-doc link".to_owned(),
                ));
            }
        }
    }

    fn owner_for(&self, item: &syn::Item) -> Option<(usize, usize)> {
        let span = find_node_span(item.span());
        let index = &self.context.indexes[self.ordinal];
        if matches!(
            item,
            syn::Item::Struct(_) | syn::Item::Enum(_) | syn::Item::Union(_) | syn::Item::Trait(_)
        ) {
            return index
                .items
                .iter()
                .position(|item| {
                    item.module == self.module && item.file == self.file && item.span == span
                })
                .map(|identity| (self.ordinal, identity));
        }
        let syn::Item::Impl(implementation) = item else {
            return None;
        };
        self.context
            .resolve_type_syntax(self.ordinal, self.module, &implementation.self_ty)
            .iter()
            .find_map(|resolved| {
                let syn::Type::Path(path) = &resolved.typed else {
                    return None;
                };
                let targets = self.context.resolve_workspace_path(
                    resolved.index,
                    resolved.module,
                    &path.path,
                );
                targets
                    .iter()
                    .find_map(|target| target.path.item.map(|identity| (target.index, identity)))
            })
    }
}

impl<'ast> Visit<'ast> for DocLinkVisitor<'_> {
    fn visit_file(&mut self, syntax: &'ast syn::File) {
        self.inspect(&syntax.attrs);
        syn::visit::visit_file(self, syntax);
    }
    fn visit_item(&mut self, item: &'ast syn::Item) {
        let owner = self.owner;
        let cfg = self.cfg.clone();
        self.cfg = cfg.and(&CfgPredicate::from_attributes(item_doc_attributes(item)));
        self.owner = self.owner_for(item);
        if !matches!(item, syn::Item::Mod(_)) {
            self.inspect(item_doc_attributes(item));
        }
        syn::visit::visit_item(self, item);
        self.owner = owner;
        self.cfg = cfg;
    }
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        let module = self.module;
        let span = find_node_span(item.span());
        if let Some(child) = self.context.indexes[self.ordinal]
            .modules
            .iter()
            .find(|child| {
                child.parent == Some(module)
                    && child
                        .module
                        .path
                        .last()
                        .is_some_and(|name| item.ident == name.as_str())
                    && child.span == span
            })
        {
            self.module = child.id;
        }
        self.inspect(&item.attrs);
        syn::visit::visit_item_mod(self, item);
        self.module = module;
    }
    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        self.inspect(&item.attrs);
        syn::visit::visit_impl_item_fn(self, item);
    }
    fn visit_impl_item_const(&mut self, item: &'ast syn::ImplItemConst) {
        self.inspect(&item.attrs);
        syn::visit::visit_impl_item_const(self, item);
    }
    fn visit_impl_item_type(&mut self, item: &'ast syn::ImplItemType) {
        self.inspect(&item.attrs);
        syn::visit::visit_impl_item_type(self, item);
    }
    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        self.inspect(&item.attrs);
        syn::visit::visit_trait_item_fn(self, item);
    }
    fn visit_trait_item_const(&mut self, item: &'ast syn::TraitItemConst) {
        self.inspect(&item.attrs);
        syn::visit::visit_trait_item_const(self, item);
    }
    fn visit_trait_item_type(&mut self, item: &'ast syn::TraitItemType) {
        self.inspect(&item.attrs);
        syn::visit::visit_trait_item_type(self, item);
    }
    fn visit_field(&mut self, item: &'ast syn::Field) {
        self.inspect(&item.attrs);
        syn::visit::visit_field(self, item);
    }
    fn visit_variant(&mut self, item: &'ast syn::Variant) {
        self.inspect(&item.attrs);
        syn::visit::visit_variant(self, item);
    }
}
