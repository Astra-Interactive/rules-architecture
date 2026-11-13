use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::declaration::util::{derives_applied, list_applied_metas};
use crate::module::util::{format_shape_subject, is_model_item, list_try_from_impls};
use crate::source::util::find_node_span;
use syn::punctuated::Punctuated;
use syn::{Meta, Token, Visibility};

pub(crate) struct ValidatedModelDerives;

impl ValidatedModelDerives {
    fn has_serde_try_from(metas: &[Meta]) -> bool {
        metas.iter().any(|meta| {
            let Meta::List(list) = meta else {
                return false;
            };
            list.path.is_ident("serde")
                && list
                    .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                    .is_ok_and(|arguments| {
                        arguments.iter().any(|argument| {
                            matches!(argument, Meta::NameValue(value) if value.path.is_ident("try_from"))
                        })
                    })
        })
    }

    fn list_bypasses(structure: &syn::ItemStruct) -> Vec<&'static str> {
        let metas = list_applied_metas(&structure.attrs);
        let mut bypasses = Vec::new();
        if derives_applied(&metas, "Deserialize") && !Self::has_serde_try_from(&metas) {
            bypasses.push(Self::DESERIALIZE_MESSAGE);
        }
        if derives_applied(&metas, "Default") {
            bypasses.push(Self::DEFAULT_MESSAGE);
        }
        bypasses
    }

    const DESERIALIZE_MESSAGE: &'static str = "A type checked by TryFrom that derives Deserialize carries #[serde(try_from = \"Raw\")]: a derived deserializer builds the value without its check (arch-domain.rs.md)";
    const DEFAULT_MESSAGE: &'static str = "A type checked by TryFrom does not derive Default: Default builds the value without its check (arch-domain.rs.md)";
}

impl Rule for ValidatedModelDerives {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            for (identity, item) in index.items.iter().enumerate() {
                let syn::Item::Struct(structure) = &item.syntax else {
                    continue;
                };
                let has_private_field = structure
                    .fields
                    .iter()
                    .any(|field| matches!(field.vis, Visibility::Inherited));
                if !is_model_item(context, ordinal, item)
                    || context.is_shape_generated_file(&item.file)
                    || !has_private_field
                    || list_try_from_impls(index, identity).is_empty()
                {
                    continue;
                }
                findings.extend(Self::list_bypasses(structure).into_iter().map(|message| {
                    build_node_finding(
                        "ValidatedModelDerives",
                        &item.file,
                        find_node_span(structure.ident.span()),
                        format_shape_subject(index, item.module, &item.name),
                        message.to_owned(),
                    )
                }));
            }
        }
        findings
    }
}
