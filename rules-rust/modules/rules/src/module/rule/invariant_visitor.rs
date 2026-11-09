use syn::visit::Visit;

pub(crate) struct InvariantVisitor {
    found: bool,
}

impl InvariantVisitor {
    pub(crate) fn calls_assert(block: &syn::Block) -> bool {
        let mut visitor = Self { found: false };
        visitor.visit_block(block);
        visitor.found
    }
}

impl<'ast> Visit<'ast> for InvariantVisitor {
    fn visit_macro(&mut self, value: &'ast syn::Macro) {
        self.found |= value.path.is_ident("assert");
        syn::visit::visit_macro(self, value);
    }
}
