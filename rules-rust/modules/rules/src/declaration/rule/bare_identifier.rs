use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::check::util::format_module_path;
use crate::declaration::util::derives;
use crate::declaration::util::is_technology;
use crate::source::util::find_node_span;
use syn::ext::IdentExt;
use syn::spanned::Spanned;

pub(crate) struct BareIdentifier;

impl BareIdentifier {
    fn is_bare(value: &syn::Type) -> bool {
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
            "String", "Uuid", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32",
            "i64", "i128", "isize",
        ]
        .contains(&name.as_str())
        {
            return true;
        }
        let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
            return false;
        };
        let Some(syn::GenericArgument::Type(inner)) = arguments.args.first() else {
            return false;
        };
        if name == "Option" {
            return Self::is_bare(inner);
        }
        matches!(name.as_str(), "Box" | "Arc")
            && matches!(inner, syn::Type::Path(path) if path.path.is_ident("str"))
    }

    fn check_fields(fields: &syn::Fields, spans: &mut Vec<proc_macro2::Span>) {
        for field in fields {
            let Some(name) = &field.ident else {
                continue;
            };
            let name = name.unraw().to_string();
            if (name == "id" || name.ends_with("_id")) && Self::is_bare(&field.ty) {
                spans.push(field.span());
            }
        }
    }

    fn check_item(item: &syn::Item, in_model: bool) -> Vec<proc_macro2::Span> {
        let mut spans = Vec::new();
        match item {
            syn::Item::Struct(value)
                if !derives(&value.attrs, "Serialize") && !derives(&value.attrs, "Deserialize") =>
            {
                let invalid_identifier = value.ident.unraw().to_string().ends_with("Id")
                    && !matches!(&value.fields, syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1);
                let single_property = in_model
                    && matches!(&value.fields, syn::Fields::Named(fields) if fields.named.len() == 1);
                if invalid_identifier || single_property {
                    spans.push(value.ident.span());
                }
                Self::check_fields(&value.fields, &mut spans);
            }
            syn::Item::Enum(value)
                if !derives(&value.attrs, "Serialize") && !derives(&value.attrs, "Deserialize") =>
            {
                for variant in &value.variants {
                    Self::check_fields(&variant.fields, &mut spans);
                }
            }
            _ => {}
        }
        spans
    }
}

impl Rule for BareIdentifier {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for index in &context.indexes {
            let contract = context.workspace.members.iter().any(|member| {
                member.name == index.crate_name && member.directory.ends_with("/contract")
            });
            for item in &index.items {
                if contract || is_technology(context, index, item.module) {
                    continue;
                }
                for span in Self::check_item(
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
