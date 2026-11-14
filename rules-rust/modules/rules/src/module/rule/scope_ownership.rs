use crate::check::model::RuleContext;
use crate::declaration::util::derives;
use crate::module::rule::{ScopeCallVisitor, ScopeOriginVisitor};
use crate::module::util::owns_implementation;
use crate::source::model::{CfgPredicate, IndexedImpl};
use syn::ImplItem;

pub(crate) struct ScopeOwnership;

impl ScopeOwnership {
    fn returns_self(typed: &syn::Type, type_name: &str) -> bool {
        let syn::Type::Path(path) = typed else {
            return false;
        };
        let Some(segment) = path.path.segments.last() else {
            return false;
        };
        if segment.ident == "Self" || segment.ident == type_name {
            return true;
        }
        let is_wrapper = ["Result", "Option", "Arc", "Box", "Rc"]
            .iter()
            .any(|wrapper| segment.ident == wrapper);
        if !is_wrapper {
            return false;
        }
        let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
            return false;
        };
        arguments
            .args
            .iter()
            .find_map(|argument| match argument {
                syn::GenericArgument::Type(inner) => Some(inner),
                _ => None,
            })
            .is_some_and(|inner| Self::returns_self(inner, type_name))
    }

    fn is_constructor(method: &syn::ImplItemFn, type_name: &str) -> bool {
        method.sig.receiver().is_none()
            && matches!(&method.sig.output, syn::ReturnType::Type(_, typed) if Self::returns_self(typed, type_name))
    }

    fn is_active(implementation: &IndexedImpl, method: &syn::ImplItemFn) -> bool {
        let is_test_only = method.attrs.iter().any(|attribute| {
            attribute.path().is_ident("cfg")
                && matches!(&attribute.meta, syn::Meta::List(list) if list.tokens.to_string() == "test")
        });
        let cfg = implementation
            .cfg
            .and(&CfgPredicate::from_attributes(&method.attrs));
        !is_test_only && cfg.is_satisfiable()
    }

    fn is_inherent_or_drop(implementation: &IndexedImpl) -> bool {
        match &implementation.syntax.trait_ {
            None => true,
            Some((path, _)) => path
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "Drop"),
        }
    }

    pub(crate) fn is_built_by_constructors(
        context: &RuleContext,
        ordinal: usize,
        identity: usize,
        field: &str,
    ) -> bool {
        let index = &context.indexes[ordinal];
        let item = &index.items[identity];
        let type_name = &item.name;
        let mut origins: Vec<bool> = Vec::new();
        for implementation in index
            .implementations
            .iter()
            .filter(|implementation| owns_implementation(index, identity, implementation))
        {
            for member in &implementation.syntax.items {
                let ImplItem::Fn(method) = member else {
                    continue;
                };
                if Self::is_active(implementation, method)
                    && Self::is_constructor(method, type_name)
                {
                    origins.extend(ScopeOriginVisitor::collect(type_name, field, &method.block));
                }
            }
        }
        if matches!(&item.syntax, syn::Item::Struct(structure) if derives(&structure.attrs, "Default"))
        {
            origins.push(true);
        }
        !origins.is_empty() && origins.iter().all(|is_owned| *is_owned)
    }

    pub(crate) fn aborts_in_owner(
        context: &RuleContext,
        ordinal: usize,
        identity: usize,
        field: &str,
    ) -> bool {
        let index = &context.indexes[ordinal];
        index
            .implementations
            .iter()
            .filter(|implementation| {
                Self::is_inherent_or_drop(implementation)
                    && owns_implementation(index, identity, implementation)
            })
            .flat_map(|implementation| &implementation.syntax.items)
            .any(|member| match member {
                ImplItem::Fn(method) => ScopeCallVisitor::collect(&method.block)
                    .iter()
                    .any(|call| call.field == field && call.method == "abort"),
                _ => false,
            })
    }
}
