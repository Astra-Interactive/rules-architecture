use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::check::util::format_module_path;
use crate::declaration::util::is_technology;
use crate::source::util::find_node_span;
use syn::ext::IdentExt;
use syn::spanned::Spanned;

pub(crate) struct BareIdentifier;

impl BareIdentifier {
    fn is_bare(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        value: &syn::Type,
        depth: usize,
    ) -> bool {
        if depth > 8 {
            return false;
        }
        context
            .resolve_type_syntax(ordinal, module, value)
            .iter()
            .any(|resolved| {
                Self::is_bare_resolved(
                    context,
                    resolved.index,
                    resolved.module,
                    &resolved.typed,
                    depth,
                )
            })
    }

    fn is_bare_resolved(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        value: &syn::Type,
        depth: usize,
    ) -> bool {
        if let syn::Type::Reference(reference) = value {
            return matches!(&*reference.elem, syn::Type::Path(path) if path.path.is_ident("str"));
        }
        let syn::Type::Path(path) = value else {
            return false;
        };
        let Some(segment) = path.path.segments.last() else {
            return false;
        };
        let name = segment.ident.unraw().to_string();
        if [
            "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
        ]
        .contains(&name.as_str())
        {
            return matches!(
                context.indexes[ordinal].resolve_path(module, &path.path),
                crate::source::model::Resolution::Local
            );
        }
        let Some(identity) = context.canonical_external_type_path(ordinal, module, &path.path)
        else {
            return false;
        };
        let segments = identity
            .segments
            .iter()
            .skip(1)
            .map(String::as_str)
            .collect::<Vec<_>>();
        let is_standard = ["std", "core", "alloc"].contains(&identity.package.as_str());
        let is_non_zero =
            matches!(segments.as_slice(), ["num", name] if name.starts_with("NonZero"));
        if (identity.package == "uuid" && segments == ["Uuid"])
            || (is_standard && (segments == ["string", "String"] || is_non_zero))
        {
            return true;
        }
        let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
            return false;
        };
        let Some(inner) = arguments.args.iter().find_map(|argument| match argument {
            syn::GenericArgument::Type(inner) => Some(inner),
            _ => None,
        }) else {
            return false;
        };
        let is_wrapper = Self::STANDARD_WRAPPERS
            .iter()
            .any(|wrapper| segments == *wrapper);
        is_standard
            && is_wrapper
            && (matches!(inner, syn::Type::Path(path) if path.path.is_ident("str"))
                || Self::is_bare(context, ordinal, module, inner, depth + 1))
    }

    fn find_collection_element<'typed>(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        value: &'typed syn::Type,
    ) -> Option<&'typed syn::Type> {
        let syn::Type::Path(path) = value else {
            return None;
        };
        let segment = path.path.segments.last()?;
        let identity = context.canonical_external_type_path(ordinal, module, &path.path)?;
        let segments = identity
            .segments
            .iter()
            .skip(1)
            .map(String::as_str)
            .collect::<Vec<_>>();
        let is_standard = ["std", "core", "alloc"].contains(&identity.package.as_str());
        let is_collection = Self::STANDARD_COLLECTIONS
            .iter()
            .any(|collection| segments.last() == Some(collection));
        if !is_standard || !is_collection {
            return None;
        }
        let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
            return None;
        };
        arguments.args.iter().find_map(|argument| match argument {
            syn::GenericArgument::Type(inner) => Some(inner),
            _ => None,
        })
    }

    fn is_bare_collection(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        value: &syn::Type,
    ) -> bool {
        context
            .resolve_type_syntax(ordinal, module, value)
            .iter()
            .any(|resolved| {
                Self::find_collection_element(
                    context,
                    resolved.index,
                    resolved.module,
                    &resolved.typed,
                )
                .is_some_and(|element| {
                    Self::is_bare(context, resolved.index, resolved.module, element, 1)
                })
            })
    }

    fn check_fields(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        fields: &syn::Fields,
        spans: &mut Vec<proc_macro2::Span>,
    ) {
        for field in fields {
            let Some(name) = &field.ident else {
                continue;
            };
            let name = name.unraw().to_string();
            let is_bare_single = (name == "id" || name.ends_with("_id"))
                && Self::is_bare(context, ordinal, module, &field.ty, 0);
            let is_bare_plural = (name == "ids" || name.ends_with("_ids"))
                && Self::is_bare_collection(context, ordinal, module, &field.ty);
            if is_bare_single || is_bare_plural {
                spans.push(field.span());
            }
        }
    }

    fn is_identity_field(field: &syn::Field) -> bool {
        let Some(name) = &field.ident else {
            return false;
        };
        let name = name.unraw().to_string();
        let is_identity_name = name == "id" || name.ends_with("_id");
        let is_typed_identifier = matches!(&field.ty, syn::Type::Path(path)
            if path.path.segments.last().is_some_and(|segment| segment.ident.to_string().ends_with("Id")));
        is_identity_name && is_typed_identifier
    }

    fn check_item(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        item: &syn::Item,
        in_model: bool,
    ) -> Vec<proc_macro2::Span> {
        let mut spans = Vec::new();
        match item {
            syn::Item::Struct(value) => {
                let invalid_identifier = value.ident.unraw().to_string().ends_with("Id")
                    && !matches!(&value.fields, syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1);
                let single_property = in_model
                    && matches!(&value.fields, syn::Fields::Named(fields)
                        if fields.named.len() == 1 && !fields.named.iter().any(Self::is_identity_field));
                if invalid_identifier || single_property {
                    spans.push(value.ident.span());
                }
                Self::check_fields(context, ordinal, module, &value.fields, &mut spans);
            }
            syn::Item::Enum(value) => {
                for variant in &value.variants {
                    Self::check_fields(context, ordinal, module, &variant.fields, &mut spans);
                }
            }
            syn::Item::Type(value)
                if in_model && value.ident.unraw().to_string().ends_with("Id") =>
            {
                spans.push(value.ident.span());
            }
            _ => {}
        }
        spans
    }

    const STANDARD_COLLECTIONS: [&'static str; 4] = ["Vec", "VecDeque", "HashSet", "BTreeSet"];

    const STANDARD_WRAPPERS: [[&'static str; 2]; 5] = [
        ["option", "Option"],
        ["boxed", "Box"],
        ["sync", "Arc"],
        ["rc", "Rc"],
        ["borrow", "Cow"],
    ];
}

impl Rule for BareIdentifier {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            let contract = context
                .find_role_config(&index.crate_name)
                .is_some_and(|config| config.role == Some(crate::config::model::Role::Contract))
                || context.workspace.members.iter().any(|member| {
                    member.name == index.crate_name && member.directory.ends_with("/contract")
                });
            for item in &index.items {
                if contract || is_technology(context, index, item.module) {
                    continue;
                }
                for span in Self::check_item(
                    context,
                    ordinal,
                    item.module,
                    &item.syntax,
                    crate::declaration::util::has_kind(index, item.module, "model"),
                ) {
                    findings.push(build_node_finding("BareIdentifier", &item.file, find_node_span(span), format!("{}::{}", format_module_path(&index.crate_name, &index.modules[item.module].module.path), item.name), "Use a single-field tuple identifier instead of bare strings, UUIDs or numbers (arch-domain.rs.md)".to_owned()));
                }
            }
        }
        findings
    }
}
