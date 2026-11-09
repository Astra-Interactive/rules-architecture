use crate::check::model::RuleContext;
use crate::source::model::Resolution;

pub(crate) fn is_result_variant(
    context: &RuleContext,
    ordinal: usize,
    module: usize,
    path: &syn::Path,
    variant: &str,
) -> bool {
    match context.indexes[ordinal].resolve_path(module, path) {
        Resolution::Local => {
            path.is_ident(variant)
                || path.segments.len() == 2
                    && path.segments[0].ident == "Result"
                    && path.segments[1].ident == variant
        }
        Resolution::External(external) => {
            let identity = context
                .resolve_external_identity(ordinal, &external)
                .format_path();
            identity == format!("std::result::Result::{variant}")
                || identity == format!("core::result::Result::{variant}")
        }
        _ => false,
    }
}
