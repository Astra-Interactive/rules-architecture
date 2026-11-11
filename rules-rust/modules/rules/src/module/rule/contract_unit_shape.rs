use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::config::model::Role;
use crate::module::util::build_shape_item_finding;
use syn::punctuated::Punctuated;
use syn::{Item, Path, Token};

pub(crate) struct ContractUnitShape;

impl ContractUnitShape {
    fn derives(attributes: &[syn::Attribute], name: &str) -> bool {
        attributes
            .iter()
            .filter(|attribute| attribute.path().is_ident("derive"))
            .filter_map(|attribute| {
                attribute
                    .parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated)
                    .ok()
            })
            .flatten()
            .any(|path| {
                let segments: Vec<_> = path.segments.iter().collect();
                match segments.as_slice() {
                    [segment] => segment.ident == name,
                    [crate_segment, segment] => {
                        crate_segment.ident == "serde" && segment.ident == name
                    }
                    _ => false,
                }
            })
    }
}

impl Rule for ContractUnitShape {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for index in &context.indexes {
            if context
                .find_role_config(&index.crate_name)
                .and_then(|config| config.role)
                != Some(Role::Contract)
            {
                continue;
            }
            for item in &index.items {
                let attrs = match &item.syntax {
                    Item::Struct(item) => Some(&item.attrs),
                    Item::Enum(item) => Some(&item.attrs),
                    _ => None,
                };
                let untyped_wire = attrs.is_some_and(|attrs| {
                    !Self::derives(attrs, "Serialize") || !Self::derives(attrs, "Deserialize")
                });
                let function_body = matches!(item.syntax, Item::Fn(_) | Item::Trait(_));
                let other_item = matches!(
                    item.syntax,
                    Item::Type(_) | Item::Union(_) | Item::Static(_)
                );
                if untyped_wire || function_body || other_item {
                    findings.push(build_shape_item_finding(index, item, "ContractUnitShape", "contract units hold Serialize + Deserialize wire types and route constants, without function bodies"));
                }
            }
            findings.extend(
                index
                    .implementations
                    .iter()
                    .filter(|implementation| {
                        implementation
                            .syntax
                            .items
                            .iter()
                            .any(|item| matches!(item, syn::ImplItem::Fn(_)))
                    })
                    .map(|implementation| {
                        build_node_finding(
                            "ContractUnitShape",
                            &implementation.file,
                            implementation.span,
                            index.modules[implementation.module].module.path.join("::"),
                            "contract units contain no function body".to_owned(),
                        )
                    }),
            );
        }
        findings
    }
}
