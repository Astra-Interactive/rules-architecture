use syn::Attribute;
use syn::Expr;
use syn::Lit;
use syn::Meta;
use syn::Token;
use syn::punctuated::Punctuated;

const TARGET_KEYS: [&str; 3] = ["target_os", "target_family", "target_arch"];
const TARGET_FAMILIES: [&str; 2] = ["unix", "windows"];
const COMBINATORS: [&str; 3] = ["all", "any", "not"];

fn find_predicate_value(predicate: &Meta) -> Option<String> {
    match predicate {
        Meta::Path(path) => TARGET_FAMILIES
            .iter()
            .find(|family| path.is_ident(family))
            .map(|family| (*family).to_owned()),
        Meta::NameValue(name_value) => {
            let is_target_key = TARGET_KEYS.iter().any(|key| name_value.path.is_ident(key));
            let Expr::Lit(literal) = &name_value.value else {
                return None;
            };
            match &literal.lit {
                Lit::Str(value) if is_target_key => Some(value.value()),
                _ => None,
            }
        }
        Meta::List(_) => None,
    }
}

fn collect_predicate_values(predicate: &Meta, values: &mut Vec<String>) {
    if let Some(value) = find_predicate_value(predicate) {
        values.push(value);
        return;
    }
    let Meta::List(list) = predicate else {
        return;
    };
    let is_combinator = COMBINATORS.iter().any(|name| list.path.is_ident(name));
    if !is_combinator {
        return;
    }
    let Ok(nested) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) else {
        return;
    };
    for inner in &nested {
        collect_predicate_values(inner, values);
    }
}

fn parse_cfg_predicate(attribute: &Attribute) -> Option<Meta> {
    if !attribute.path().is_ident("cfg") {
        return None;
    }
    attribute.parse_args::<Meta>().ok()
}

pub(crate) fn find_single_target_value(attributes: &[Attribute]) -> Option<String> {
    attributes
        .iter()
        .filter_map(parse_cfg_predicate)
        .find_map(|predicate| find_predicate_value(&predicate))
}

pub(crate) fn list_target_values(attribute: &Attribute) -> Vec<String> {
    let mut values = Vec::new();
    if let Some(predicate) = parse_cfg_predicate(attribute) {
        collect_predicate_values(&predicate, &mut values);
    }
    values
}

pub(crate) fn list_manifest_target_values(text: &str) -> Vec<String> {
    let mut values = Vec::new();
    let Ok(Meta::List(cfg)) = syn::parse_str::<Meta>(text) else {
        return values;
    };
    if !cfg.path.is_ident("cfg") {
        return values;
    }
    if let Ok(predicate) = cfg.parse_args::<Meta>() {
        collect_predicate_values(&predicate, &mut values);
    }
    values
}
