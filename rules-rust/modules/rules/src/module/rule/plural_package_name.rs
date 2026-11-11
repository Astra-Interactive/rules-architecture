use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::util::{build_shape_module_finding, list_shape_words};

pub(crate) struct PluralPackageName;

impl PluralPackageName {
    const SINGULAR_AS_WORDS: [&'static str; 7] = [
        "alias",
        "atlas",
        "bias",
        "canvas",
        "christmas",
        "gas",
        "pancreas",
    ];
}

impl Rule for PluralPackageName {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context.indexes.iter().enumerate().flat_map(|(ordinal, index)| {
            let exceptions = list_shape_words(context, &index.crate_name, "plural-concepts");
            index.modules.iter().filter(|module| !module.is_file_module).filter_map(move |module| {
                let name = module.module.path.last()?;
                let plural = name.ends_with('s') && !["ss", "us", "is"].iter().any(|suffix| name.ends_with(suffix));
                if !plural || ["news", "settings"].contains(&name.as_str()) || Self::SINGULAR_AS_WORDS.contains(&name.as_str()) || exceptions.contains(name) || context.is_technology_kind(&index.crate_name, name) { return None; }
                build_shape_module_finding(context, ordinal, module.id, "PluralPackageName", "name a concept in the singular, or declare its project word in plural-concepts")
            }).collect::<Vec<_>>()
        }).collect()
    }
}
