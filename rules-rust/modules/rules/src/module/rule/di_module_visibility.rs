use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::module::util::is_di_type;

pub(crate) struct DiModuleVisibility;

impl Rule for DiModuleVisibility {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .indexes
            .iter()
            .enumerate()
            .filter(|(ordinal, _)| context.can_prove_workspace_only(*ordinal))
            .flat_map(|(ordinal, index)| {
                index
                    .items
                    .iter()
                    .enumerate()
                    .filter(move |(identity, item)| {
                        RuleContext::is_public_item(item)
                            && is_di_type(context, ordinal, item)
                            && !context.has_external_name_user(ordinal, *identity)
                    })
                    .map(|(_, item)| {
                        build_node_finding(
                            "DiModuleVisibility",
                            &item.file,
                            item.span,
                            item.name.clone(),
                            "keep a DI module within its crate unless another crate names it"
                                .to_owned(),
                        )
                    })
            })
            .collect()
    }
}
