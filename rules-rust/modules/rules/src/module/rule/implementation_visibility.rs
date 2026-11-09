use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::util::{build_shape_item_finding, find_kind_name, implements_crate_trait};
use syn::{Item, Visibility};

pub(crate) struct ImplementationVisibility;

impl ImplementationVisibility {
    fn check_workspace(context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            for (identity, item) in
                index.items.iter().enumerate().filter(|(_, item)| {
                    RuleContext::is_public_item(item) && item.cfg.is_satisfiable()
                })
            {
                let kind = find_kind_name(context, ordinal, item.module);
                let storage = ["table", "entity", "database"].contains(&kind);
                let private_implementation = ["stub", "memory"].contains(&kind)
                    && context.can_prove_workspace_only(ordinal)
                    && !context.has_external_name_user(ordinal, identity)
                    && matches!(item.syntax, Item::Struct(_))
                    && index
                        .implementations
                        .iter()
                        .any(|implementation| implements_crate_trait(index, item, implementation));
                if storage || private_implementation {
                    findings.push(build_shape_item_finding(index,item,"ImplementationVisibility","storage details and implementations without external consumers remain within the crate"));
                }
            }
        }
        findings
    }
}

impl Rule for ImplementationVisibility {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings: Vec<_> = context
            .indexes
            .iter()
            .enumerate()
            .flat_map(|(ordinal, index)| {
                index.items.iter().filter_map(move |item| {
                    let Item::Struct(structure) = &item.syntax else {
                        return None;
                    };
                    if !matches!(structure.vis, Visibility::Public(_)) {
                        return None;
                    }
                    let kind = find_kind_name(context, ordinal, item.module);
                    if ["stub", "memory"].contains(&kind)
                        || !context.is_technology_kind(&index.crate_name, kind)
                    {
                        return None;
                    }
                    index
                        .implementations
                        .iter()
                        .any(|implementation| implements_crate_trait(index, item, implementation))
                        .then(|| {
                            build_shape_item_finding(
                                index,
                                item,
                                "ImplementationVisibility",
                                "a technology implementation of a crate contract is pub(crate)",
                            )
                        })
                })
            })
            .collect();
        findings.extend(Self::check_workspace(context));
        findings
    }
}
