use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::comment::util::item_doc_attributes;
use crate::source::model::{CfgPredicate, SourceFile};
use crate::source::util::find_node_span;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct DocHiddenVisitor<'source> {
    file: &'source SourceFile,
    cfg: CfgPredicate,
    exposed: bool,
    inherited_fields: bool,
    trait_impl: bool,
    findings: Vec<RuleFinding>,
}

impl DocHiddenVisitor<'_> {
    fn is_public_item(item: &syn::Item) -> bool {
        let visibility = match item {
            syn::Item::Const(item) => &item.vis,
            syn::Item::Enum(item) => &item.vis,
            syn::Item::ExternCrate(item) => &item.vis,
            syn::Item::Fn(item) => &item.vis,
            syn::Item::Mod(item) => &item.vis,
            syn::Item::Static(item) => &item.vis,
            syn::Item::Struct(item) => &item.vis,
            syn::Item::Trait(item) => &item.vis,
            syn::Item::TraitAlias(item) => &item.vis,
            syn::Item::Type(item) => &item.vis,
            syn::Item::Union(item) => &item.vis,
            syn::Item::Use(item) => &item.vis,
            _ => return false,
        };
        matches!(visibility, syn::Visibility::Public(_))
    }

    fn meta_hides(meta: &syn::Meta) -> bool {
        let syn::Meta::List(list) = meta else {
            return false;
        };
        let Ok(entries) =
            list.parse_args_with(Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated)
        else {
            return false;
        };
        if list.path.is_ident("doc") {
            return entries.iter().any(|entry| entry.path().is_ident("hidden"));
        }
        list.path.is_ident("cfg_attr") && entries.iter().skip(1).any(Self::meta_hides)
    }

    pub(crate) fn hides(attribute: &syn::Attribute) -> bool {
        Self::meta_hides(&attribute.meta)
    }

    fn inspect(&mut self, attributes: &[syn::Attribute], exposed: bool) {
        if exposed {
            return;
        }
        let active = self
            .cfg
            .and(&CfgPredicate::from_attributes(attributes))
            .is_satisfiable();
        if !active {
            return;
        }
        for attribute in attributes.iter().filter(|attribute| Self::hides(attribute)) {
            self.findings.push(build_node_finding(
                "DocHiddenUse",
                &self.file.path,
                find_node_span(attribute.span()),
                self.file.path.clone(),
                "reserve doc(hidden) public visibility for actual consumers in another crate"
                    .to_owned(),
            ));
        }
    }

    fn inspect_impl_member(&mut self, attributes: &[syn::Attribute], visibility: &syn::Visibility) {
        let exposed = self.trait_impl || matches!(visibility, syn::Visibility::Public(_));
        self.inspect(attributes, exposed);
    }

    pub(crate) fn collect(file: &SourceFile, cfg: CfgPredicate) -> Vec<RuleFinding> {
        let mut visitor = DocHiddenVisitor {
            file,
            cfg,
            exposed: false,
            inherited_fields: false,
            trait_impl: false,
            findings: Vec::new(),
        };
        visitor.visit_file(&file.syntax);
        visitor.findings
    }
}

impl<'ast> Visit<'ast> for DocHiddenVisitor<'_> {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        let cfg = self.cfg.clone();
        let exposed = self.exposed;
        let inherited_fields = self.inherited_fields;
        self.cfg = cfg.and(&CfgPredicate::from_attributes(item_doc_attributes(item)));
        self.exposed = Self::is_public_item(item);
        self.inherited_fields = matches!(item, syn::Item::Enum(_));
        self.inspect(item_doc_attributes(item), self.exposed);
        if self.cfg.is_satisfiable() {
            syn::visit::visit_item(self, item);
        }
        self.cfg = cfg;
        self.exposed = exposed;
        self.inherited_fields = inherited_fields;
    }

    fn visit_field(&mut self, field: &'ast syn::Field) {
        self.inspect(
            &field.attrs,
            self.exposed
                && (self.inherited_fields || matches!(field.vis, syn::Visibility::Public(_))),
        );
        syn::visit::visit_field(self, field);
    }

    fn visit_variant(&mut self, variant: &'ast syn::Variant) {
        let cfg = self.cfg.clone();
        self.cfg = cfg.and(&CfgPredicate::from_attributes(&variant.attrs));
        self.inspect(&variant.attrs, self.exposed);
        if self.cfg.is_satisfiable() {
            syn::visit::visit_variant(self, variant);
        }
        self.cfg = cfg;
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let trait_impl = self.trait_impl;
        self.trait_impl = item.trait_.is_some();
        syn::visit::visit_item_impl(self, item);
        self.trait_impl = trait_impl;
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        self.inspect_impl_member(&item.attrs, &item.vis);
        syn::visit::visit_impl_item_fn(self, item);
    }

    fn visit_impl_item_const(&mut self, item: &'ast syn::ImplItemConst) {
        self.inspect_impl_member(&item.attrs, &item.vis);
        syn::visit::visit_impl_item_const(self, item);
    }

    fn visit_impl_item_type(&mut self, item: &'ast syn::ImplItemType) {
        self.inspect_impl_member(&item.attrs, &item.vis);
        syn::visit::visit_impl_item_type(self, item);
    }

    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        self.inspect(&item.attrs, self.exposed);
        syn::visit::visit_trait_item_fn(self, item);
    }

    fn visit_trait_item_type(&mut self, item: &'ast syn::TraitItemType) {
        self.inspect(&item.attrs, self.exposed);
        syn::visit::visit_trait_item_type(self, item);
    }

    fn visit_trait_item_const(&mut self, item: &'ast syn::TraitItemConst) {
        self.inspect(&item.attrs, self.exposed);
        syn::visit::visit_trait_item_const(self, item);
    }
}
