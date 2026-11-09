use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::module::util::{
    build_shape_item_finding, external_trait, find_kind_name, owns_implementation,
};
use crate::source::model::IndexedItem;

pub(crate) struct ListenerName;

impl ListenerName {
    fn check_item(
        context: &RuleContext,
        ordinal: usize,
        identity: usize,
        item: &IndexedItem,
    ) -> Vec<RuleFinding> {
        let index = &context.indexes[ordinal];
        let mut findings = Vec::new();
        if !matches!(item.syntax, syn::Item::Struct(_)) {
            return findings;
        }
        let external = index
            .implementations
            .iter()
            .filter(|implementation| owns_implementation(index, identity, implementation))
            .filter_map(|implementation| {
                external_trait(context, ordinal, implementation).map(|path| (implementation, path))
            })
            .filter(|(_, path)| {
                path.rsplit("::")
                    .next()
                    .is_some_and(|name| name.ends_with("Listener"))
            })
            .collect::<Vec<_>>();
        if external.is_empty() {
            return findings;
        }
        if find_kind_name(context, ordinal, item.module) == "event"
            && (!item.name.ends_with("Listener") || item.name.ends_with("Event"))
        {
            findings.push(build_shape_item_finding(index, item, "ListenerName", "An event listener implementation is named <Subject>Listener, never Event (arch-domain.md)"));
        }
        for (implementation, _) in external {
            if find_kind_name(context, ordinal, implementation.module) != "event" {
                findings.push(build_node_finding(
                    "ListenerName",
                    &implementation.file,
                    implementation.span,
                    item.name.clone(),
                    "Implement a platform listener in the event kind (no-callbacks.rs.md)"
                        .to_owned(),
                ));
            }
        }
        findings
    }
}

impl Rule for ListenerName {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .indexes
            .iter()
            .enumerate()
            .flat_map(|(ordinal, index)| {
                index
                    .items
                    .iter()
                    .enumerate()
                    .flat_map(move |(identity, item)| {
                        Self::check_item(context, ordinal, identity, item)
                    })
            })
            .collect()
    }
}
