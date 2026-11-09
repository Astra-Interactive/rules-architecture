use std::collections::BTreeMap;

#[derive(Clone)]
pub(crate) struct TypeBinding {
    pub(crate) index: usize,
    pub(crate) module: usize,
    pub(crate) typed: syn::Type,
    pub(crate) parameters: BTreeMap<String, TypeBinding>,
}
