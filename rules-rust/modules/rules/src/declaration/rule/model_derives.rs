use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::check::util::format_module_path;
use crate::declaration::util::derives;
use crate::declaration::util::has_kind;
use crate::declaration::util::is_technology;
use crate::source::util::find_node_span;

pub(crate) struct ModelDerives;

impl Rule for ModelDerives {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            for item in &index.items {
                let syn::Item::Struct(value) = &item.syntax else {
                    continue;
                };
                if !has_kind(index, item.module, "model")
                    || is_technology(context, index, item.module)
                {
                    continue;
                }
                let mut required = vec!["Debug", "Clone", "PartialEq"];
                if item.name.ends_with("Id")
                    || value.fields.iter().all(|field| {
                        crate::declaration::util::supports_eq_hash(
                            context,
                            ordinal,
                            item.module,
                            &field.ty,
                            0,
                        )
                    })
                {
                    required.extend(["Eq", "Hash"]);
                }
                let missing = required
                    .into_iter()
                    .filter(|name| !derives(&value.attrs, name))
                    .collect::<Vec<_>>();
                if !missing.is_empty() {
                    findings.push(build_node_finding(
                        "ModelDerives",
                        &item.file,
                        find_node_span(value.ident.span()),
                        format!(
                            "{}::{}",
                            format_module_path(
                                &index.crate_name,
                                &index.modules[item.module].module.path
                            ),
                            item.name
                        ),
                        format!(
                            "Derive {} on this model struct (arch-domain.rs.md)",
                            missing.join(", ")
                        ),
                    ));
                }
            }
        }
        findings
    }
}
