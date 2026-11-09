use crate::source::model::CfgPredicate;

pub(crate) struct ResolvedTypeSyntax {
    pub(crate) index: usize,
    pub(crate) module: usize,
    pub(crate) typed: syn::Type,
    pub(crate) cfg: CfgPredicate,
    pub(crate) has_unsubstituted_parameters: bool,
}
