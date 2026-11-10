use syn::visit::Visit;

pub(crate) struct InvariantVisitor {
    family: bool,
    found: bool,
}

impl InvariantVisitor {
    pub(crate) fn calls_assert(block: &syn::Block) -> bool {
        let mut visitor = Self {
            family: false,
            found: false,
        };
        visitor.visit_block(block);
        visitor.found
    }

    pub(crate) fn calls_assertion(block: &syn::Block) -> bool {
        let mut visitor = Self {
            family: true,
            found: false,
        };
        visitor.visit_block(block);
        visitor.found
    }

    pub(crate) fn is_assertion(path: &syn::Path) -> bool {
        let names = [
            "assert",
            "assert_eq",
            "assert_ne",
            "debug_assert",
            "debug_assert_eq",
            "debug_assert_ne",
        ];
        let Some(last) = path.segments.last() else {
            return false;
        };
        let rooted = path.segments.len() == 1
            || (path.segments.len() == 2
                && path
                    .segments
                    .first()
                    .is_some_and(|root| root.ident == "std" || root.ident == "core"));
        rooted && names.iter().any(|name| last.ident == name)
    }
}

impl<'ast> Visit<'ast> for InvariantVisitor {
    fn visit_macro(&mut self, value: &'ast syn::Macro) {
        self.found |= if self.family {
            Self::is_assertion(&value.path)
        } else {
            value.path.is_ident("assert")
        };
        syn::visit::visit_macro(self, value);
    }
}
