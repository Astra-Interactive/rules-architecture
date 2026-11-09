use syn::visit::Visit;

pub(crate) struct PlacementTypeVisitor {
    paths: Vec<syn::Path>,
}

impl PlacementTypeVisitor {
    pub(crate) fn collect_paths(value: &syn::Type) -> Vec<syn::Path> {
        let mut visitor = Self { paths: Vec::new() };
        visitor.visit_type(value);
        visitor.paths
    }
}

impl<'ast> Visit<'ast> for PlacementTypeVisitor {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        self.paths.push(path.clone());
        syn::visit::visit_path(self, path);
    }
}
