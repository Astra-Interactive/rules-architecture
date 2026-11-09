use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use quote::ToTokens;

pub(crate) struct DocHiddenUse;

impl Rule for DocHiddenUse {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context
            .indexes
            .iter()
            .enumerate()
            .filter(|(ordinal, _)| context.can_prove_workspace_only(*ordinal))
        {
            for (identity, item) in index
                .items
                .iter()
                .enumerate()
                .filter(|(_, item)| RuleContext::is_public_item(item))
            {
                let attrs = match &item.syntax {
                    syn::Item::Struct(item) => &item.attrs,
                    syn::Item::Enum(item) => &item.attrs,
                    syn::Item::Trait(item) => &item.attrs,
                    syn::Item::Type(item) => &item.attrs,
                    syn::Item::Fn(item) => &item.attrs,
                    _ => continue,
                };
                if attrs.iter().any(|attribute| {
                    attribute.path().is_ident("doc")
                        && attribute.meta.to_token_stream().to_string() == "doc (hidden)"
                }) && !context.has_external_name_user(ordinal, identity)
                {
                    findings.push(build_node_finding("DocHiddenUse",&item.file,item.span,item.name.clone(),"reserve doc(hidden) public visibility for actual consumers in another crate".to_owned()));
                }
            }
        }
        findings
    }
}
