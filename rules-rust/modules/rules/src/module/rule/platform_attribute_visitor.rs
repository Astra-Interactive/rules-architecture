use crate::source::util::list_target_values;
use crate::source::util::parse_macro_arguments;
use std::collections::BTreeSet;
use syn::Attribute;
use syn::visit::Visit;

pub(crate) struct PlatformAttributeVisitor {
    pub(crate) values: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for PlatformAttributeVisitor {
    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        for argument in &parse_macro_arguments(item) {
            self.visit_expr(argument);
        }
    }

    fn visit_attribute(&mut self, attribute: &'ast Attribute) {
        self.values.extend(list_target_values(attribute));
        syn::visit::visit_attribute(self, attribute);
    }
}
