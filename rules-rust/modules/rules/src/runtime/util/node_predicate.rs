use crate::source::model::{CfgPredicate, CrateIndex, NodeSpan};
use crate::source::util::find_node_span;
use syn::spanned::Spanned;

pub(crate) fn node_predicate(
    index: &CrateIndex,
    module: usize,
    file: &str,
    span: NodeSpan,
) -> CfgPredicate {
    let contains = |outer: NodeSpan| outer.start <= span.start && outer.end >= span.end;
    let mut cfg = index.modules[module].cfg.clone();
    for item in &index.items {
        if item.file != file || !contains(item.span) {
            continue;
        }
        cfg = cfg.and(&item.cfg);
        let fields = match &item.syntax {
            syn::Item::Struct(item) => Some(&item.fields),
            _ => None,
        };
        if let Some(fields) = fields {
            for field in fields {
                if contains(find_node_span(field.span())) {
                    cfg = cfg.and(&CfgPredicate::from_attributes(&field.attrs));
                }
            }
        }
    }
    for implementation in &index.implementations {
        if implementation.file != file || !contains(implementation.span) {
            continue;
        }
        cfg = cfg.and(&implementation.cfg);
        for item in &implementation.syntax.items {
            if let syn::ImplItem::Fn(method) = item
                && contains(find_node_span(method.span()))
            {
                cfg = cfg.and(&CfgPredicate::from_attributes(&method.attrs));
            }
        }
    }
    cfg
}
