use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::check::util::format_module_path;
use crate::module::util::{
    build_shape_item_finding, build_shape_module_finding, find_kind_name, resolve_shape_type,
};
use crate::source::model::Resolution;
use crate::syntax::util::to_snake_case;
use std::collections::BTreeSet;

pub(crate) struct UtilFileName;

impl UtilFileName {
    fn list_extension_types(
        index: &crate::source::model::CrateIndex,
        module: usize,
    ) -> BTreeSet<ExtensionType> {
        let mut types = BTreeSet::new();
        for implementation in index
            .implementations
            .iter()
            .filter(|implementation| implementation.module == module)
        {
            types.extend(
                resolve_shape_type(index, module, &implementation.syntax.self_ty)
                    .iter()
                    .map(|target| ExtensionType {
                        identity: target.to_string(),
                        name: index.items[*target].name.clone(),
                    }),
            );
            let syn::Type::Path(path) = implementation.syntax.self_ty.as_ref() else {
                continue;
            };
            if let Resolution::External(path) = index.resolve_path(module, &path.path) {
                types.insert(ExtensionType {
                    name: path.rsplit("::").next().unwrap_or("").to_owned(),
                    identity: path,
                });
            }
        }
        types
    }

    fn check_file(context: &RuleContext, ordinal: usize, module: usize) -> Option<RuleFinding> {
        let index = &context.indexes[ordinal];
        let current = &index.modules[module];
        if context.is_shape_generated_file(&current.file) {
            return None;
        }
        if !current.is_file_module || find_kind_name(context, ordinal, module) != "util" {
            return None;
        }
        let name = current.module.path.last().map(String::as_str).unwrap_or("");
        let types = Self::list_extension_types(index, module);
        let free_function = index
            .items
            .iter()
            .any(|item| item.module == module && matches!(item.syntax, syn::Item::Fn(_)));
        let wrong_extension = types.len() > 1
            || types
                .iter()
                .any(|ty| name != format!("{}_ext", to_snake_case(&ty.name)));
        if !(wrong_extension || free_function && name.ends_with("_ext")) {
            return None;
        }
        build_shape_module_finding(
            context,
            ordinal,
            module,
            "UtilFileName",
            "one type's extension blocks live in <type>_ext.rs; a free-function file names its operation",
        )
    }

    fn check_trait(
        context: &RuleContext,
        ordinal: usize,
        item: &crate::source::model::IndexedItem,
    ) -> Option<RuleFinding> {
        if context.is_shape_generated_file(&item.file) {
            return None;
        }
        if !matches!(item.syntax, syn::Item::Trait(_))
            || find_kind_name(context, ordinal, item.module) != "util"
        {
            return None;
        }
        let owner = item.name.strip_suffix("Ext")?;
        let typed = syn::parse_str::<syn::Type>(owner).ok()?;
        let index = &context.indexes[ordinal];
        let own_type = index
            .resolve_type_items(item.module, &typed)
            .iter()
            .any(|target| {
                matches!(
                    index.items[*target].syntax,
                    syn::Item::Struct(_) | syn::Item::Enum(_) | syn::Item::Union(_)
                )
            });
        own_type.then(|| {
            build_shape_item_finding(
                index,
                item,
                "UtilFileName",
                "a crate type uses an inherent impl, never an extension trait",
            )
        })
    }

    fn check_extensions(context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            findings.extend(
                index
                    .modules
                    .iter()
                    .filter_map(|module| Self::check_file(context, ordinal, module.id)),
            );
            findings.extend(
                index
                    .items
                    .iter()
                    .filter_map(|item| Self::check_trait(context, ordinal, item)),
            );
        }
        findings
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct ExtensionType {
    identity: String,
    name: String,
}

impl Rule for UtilFileName {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings: Vec<_> = context.crates.iter().flat_map(|sources| sources.modules.iter().filter(|module| {
            !sources.files.iter().any(|file| file.module == module.id && file.is_generated)
        })).filter_map(|module| {
            let declaration = module.declaration.as_ref()?;
            let name = module.id.path.last()?;
            if declaration.is_inline || !["utils", "helpers", "extensions"].contains(&name.as_str()) {
                return None;
            }
            Some(RuleFinding {
                finding: Finding {
                    rule_id: RuleId("UtilFileName".to_owned()),
                    location: Location { path: declaration.file.clone(), line: Some(declaration.span.start.line) },
                    subject: format_module_path(&module.id.crate_name, &module.id.path),
                    message: format!("`{name}.rs` does not name what its functions do (arch-package.rs.md, What goes in one file)"),
                },
                anchor: Anchor::Module { module: module.id.clone(), file: declaration.file.clone(), body_file: declaration.body_file.clone(), declaration: declaration.span },
            })
        }).collect();
        findings.extend(Self::check_extensions(context));
        findings
    }
}
