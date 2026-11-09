use syn::GenericArgument;
use syn::Item;
use syn::PathArguments;
use syn::ReturnType;
use syn::Signature;
use syn::Type;

pub(crate) fn find_type_name(item: &Item) -> Option<&syn::Ident> {
    match item {
        Item::Struct(item) => Some(&item.ident),
        Item::Enum(item) => Some(&item.ident),
        Item::Union(item) => Some(&item.ident),
        Item::Trait(item) => Some(&item.ident),
        _ => None,
    }
}

pub(crate) fn find_self_name(self_type: &Type) -> Option<String> {
    let Type::Path(path) = self_type else {
        return None;
    };
    path.path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
}

fn returns_self(value: &Type, own_name: &str) -> bool {
    let Type::Path(path) = value else {
        return false;
    };
    path.path.segments.last().is_some_and(|segment| {
        if segment.ident == "Self" || segment.ident == own_name { return true; }
        if segment.ident != "Result" && segment.ident != "Option" { return false; }
        let PathArguments::AngleBracketed(arguments) = &segment.arguments else { return false; };
        matches!(arguments.args.first(), Some(GenericArgument::Type(value)) if returns_self(value, own_name))
    })
}

pub(crate) fn is_constructor(signature: &Signature, own_name: &str) -> bool {
    if signature.receiver().is_some() {
        return false;
    }
    let name = signature.ident.to_string();
    if name == "new" || name.starts_with("with_") || name.starts_with("from_") {
        return true;
    }
    matches!(&signature.output, ReturnType::Type(_, value) if returns_self(value, own_name))
}

pub(crate) fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    let characters: Vec<char> = name.chars().collect();
    for (index, character) in characters.iter().enumerate() {
        let previous_lower = index > 0
            && (characters[index - 1].is_lowercase() || characters[index - 1].is_ascii_digit());
        let ends_acronym = index > 0
            && characters
                .get(index + 1)
                .is_some_and(|next| next.is_lowercase());
        if character.is_uppercase() && (previous_lower || ends_acronym) {
            result.push('_');
        }
        result.extend(character.to_lowercase());
    }
    result
}
