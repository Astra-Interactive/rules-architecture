use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::source::util::parse_macro_arguments;
use crate::source::util::{find_node_span, format_tokens};
use syn::visit::Visit;
use syn::{Attribute, ItemMod};

pub(crate) struct TestAttributeVisitor {
    pub(crate) file: String,
    pub(crate) subject: String,
    pub(crate) is_root: bool,
    pub(crate) in_test: bool,
    pub(crate) findings: Vec<RuleFinding>,
}

impl<'ast> Visit<'ast> for TestAttributeVisitor {
    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        for argument in &parse_macro_arguments(item) {
            self.visit_expr(argument);
        }
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        if self.is_root && (module.ident == "test" || module.ident == "fixture") {
            let was_test = self.in_test;
            self.in_test = true;
            self.is_root = false;
            if let Some((_, items)) = &module.content {
                for item in items {
                    self.visit_item(item);
                }
            }
            self.is_root = true;
            self.in_test = was_test;
            return;
        }
        let was_root = self.is_root;
        self.is_root = false;
        syn::visit::visit_item_mod(self, module);
        self.is_root = was_root;
    }

    fn visit_attribute(&mut self, attribute: &'ast Attribute) {
        let tokens = format_tokens(&attribute.meta).replace(' ', "");
        let is_test = attribute.path().is_ident("test") || tokens == "cfg(test)";
        if is_test && !self.in_test || attribute.path().is_ident("path") {
            self.findings.push(build_node_finding("TestMirror", &self.file,
                find_node_span(attribute.pound_token.span), self.subject.clone(),
                "tests live in the root test block; path attributes belong only to the root test and fixture blocks".to_owned()));
        }
        syn::visit::visit_attribute(self, attribute);
    }
}
