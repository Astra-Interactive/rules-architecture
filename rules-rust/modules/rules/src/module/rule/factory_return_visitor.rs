use crate::source::model::{CrateIndex, Resolution};
use syn::visit::Visit;

pub(crate) struct FactoryReturnVisitor<'a> {
    index: &'a CrateIndex,
    module: usize,
    product: usize,
    pub(crate) uses_product: bool,
}

impl<'a> FactoryReturnVisitor<'a> {
    pub(crate) fn new(index: &'a CrateIndex, module: usize, product: usize) -> Self {
        Self {
            index,
            module,
            product,
            uses_product: false,
        }
    }
}

impl<'ast> Visit<'ast> for FactoryReturnVisitor<'_> {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        if matches!(self.index.resolve_path(self.module, path), Resolution::Internal(targets) if targets.iter().any(|target| target.item == Some(self.product)))
        {
            self.uses_product = true;
        }
        syn::visit::visit_path(self, path);
    }
}
