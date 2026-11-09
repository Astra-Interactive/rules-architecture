use crate::check::api::Rule;
use crate::check::model::Anchor;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::check::util::format_module_path;
use crate::family::model::TypeBehaviour;
use crate::family::model::TypeDeclaration;
use crate::family::util::find_primary_type;
use crate::family::util::uses_type;
use crate::source::model::CrateSources;
use crate::source::model::SourceFile;
use crate::source::util::find_node_span;
use crate::syntax::util::find_self_name;
use crate::syntax::util::find_type_name;
use std::collections::HashMap;
use std::collections::HashSet;
use syn::ext::IdentExt;
use syn::spanned::Spanned;

pub(crate) struct OneTypeFamilyPerFile;

impl OneTypeFamilyPerFile {
    fn matches_type_path(
        path: &syn::Path,
        scope: &[String],
        candidate: &SourceFile,
        name: &str,
    ) -> bool {
        let mut resolved = if path
            .segments
            .first()
            .is_some_and(|segment| segment.ident == "crate")
        {
            Vec::new()
        } else {
            scope.to_vec()
        };
        for segment in &path.segments {
            match segment.ident.unraw().to_string().as_str() {
                "crate" | "self" => {}
                "super" => {
                    resolved.pop();
                }
                part => resolved.push(part.to_owned()),
            }
        }
        let mut declared = candidate.module.path.clone();
        declared.push(name.to_owned());
        if resolved == declared {
            return true;
        }
        declared = candidate.module.path.clone();
        declared.pop();
        declared.push(name.to_owned());
        resolved == declared
    }

    fn find_behaviour_owner(
        value: &syn::ItemImpl,
        items: &[syn::Item],
        scope: &[String],
        origin: &SourceFile,
        files: &[SourceFile],
    ) -> Option<TypeBehaviour> {
        let syn::Type::Path(self_type) = value.self_ty.as_ref() else {
            return None;
        };
        if self_type.qself.is_some() || self_type.path.leading_colon.is_some() {
            return None;
        }
        let name = find_self_name(&value.self_ty)?;
        let candidates: Vec<&SourceFile> = files
            .iter()
            .filter(|file| {
                file.syntax
                    .items
                    .iter()
                    .any(|item| find_type_name(item).is_some_and(|ident| ident == name.as_str()))
            })
            .collect();
        let is_bare = self_type.path.leading_colon.is_none() && self_type.path.segments.len() == 1;
        if is_bare {
            let is_local = items
                .iter()
                .any(|item| find_type_name(item).is_some_and(|ident| ident == name.as_str()));
            if is_local {
                return (scope == origin.module.path).then(|| TypeBehaviour {
                    file: origin.path.clone(),
                    name,
                });
            }
            return if candidates.len() == 1 {
                candidates.first().map(|owner| TypeBehaviour {
                    file: owner.path.clone(),
                    name,
                })
            } else {
                None
            };
        }
        let mut matching = candidates
            .into_iter()
            .filter(|candidate| Self::matches_type_path(&self_type.path, scope, candidate, &name));
        let owner = matching.next()?;
        matching.next().is_none().then(|| TypeBehaviour {
            file: owner.path.clone(),
            name,
        })
    }

    fn collect_behaviour(
        items: &[syn::Item],
        scope: &[String],
        origin: &SourceFile,
        files: &[SourceFile],
        names: &mut HashMap<String, HashSet<String>>,
    ) {
        for item in items {
            match item {
                syn::Item::Impl(value) => {
                    let has_function = value
                        .items
                        .iter()
                        .any(|child| matches!(child, syn::ImplItem::Fn(_)));
                    if !has_function {
                        continue;
                    }
                    let Some(owner) =
                        Self::find_behaviour_owner(value, items, scope, origin, files)
                    else {
                        continue;
                    };
                    names.entry(owner.file).or_default().insert(owner.name);
                }
                syn::Item::Mod(module) => {
                    if let Some((_, children)) = &module.content {
                        let mut child_scope = scope.to_vec();
                        child_scope.push(module.ident.unraw().to_string());
                        Self::collect_behaviour(children, &child_scope, origin, files, names);
                    }
                }
                _ => {}
            }
        }
    }

    fn build_missing_primary(file: &SourceFile, sources: &CrateSources) -> Vec<RuleFinding> {
        sources
            .modules
            .iter()
            .filter(|module| module.id == file.module)
            .filter_map(|module| module.declaration.as_ref())
            .filter(|declaration| !declaration.is_inline && declaration.body_file == file.path)
            .map(|declaration| {
                let path = declaration.file.as_str();
                let span = declaration.span;
                let finding = build_node_finding(
                    "OneTypeFamilyPerFile",
                    path,
                    span,
                    format_module_path(&file.module.crate_name, &file.module.path),
                    "Name the file after its primary type family (arch-package.rs.md)".to_owned(),
                );
                RuleFinding {
                    anchor: Anchor::Module {
                        module: file.module.clone(),
                        file: path.to_owned(),
                        body_file: file.path.clone(),
                        declaration: span,
                    },
                    ..finding
                }
            })
            .collect()
    }

    fn check_file(
        file: &SourceFile,
        sources: &CrateSources,
        behaviour: &HashSet<String>,
    ) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        let types: Vec<TypeDeclaration<'_>> = file
            .syntax
            .items
            .iter()
            .filter_map(|item| find_type_name(item).map(|name| TypeDeclaration { item, name }))
            .collect();
        if types.is_empty() {
            return findings;
        }
        let primary = find_primary_type(file, sources);
        let subject = format_module_path(&file.module.crate_name, &file.module.path);
        if primary.is_none() {
            findings.extend(Self::build_missing_primary(file, sources));
        }
        let primary_name = primary.and_then(find_type_name).map(ToString::to_string);
        for (index, item) in types.iter().enumerate() {
            if primary_name.is_none() {
                break;
            }
            let name = item.name.to_string();
            if primary_name.as_ref() == Some(&name) {
                continue;
            }
            let is_factory = matches!(item.item, syn::Item::Trait(_))
                && primary_name
                    .as_ref()
                    .is_some_and(|primary| name == format!("{primary}Factory"));
            if is_factory {
                continue;
            }
            let user = types
                .iter()
                .position(|candidate| uses_type(candidate.item, &name));
            let invalid = behaviour.contains(&name)
                || matches!(item.item, syn::Item::Trait(_))
                || user.is_none()
                || user.is_some_and(|position| index < position);
            if invalid {
                findings.push(build_node_finding("OneTypeFamilyPerFile", &file.path, find_node_span(item.item.span()), format!("{subject}::{name}"), "Keep only the primary type and plain data types declared after their users in its file (arch-package.rs.md)".to_owned()));
            }
        }
        for item in &file.syntax.items {
            if let syn::Item::Fn(function) = item {
                findings.push(build_node_finding("OneTypeFamilyPerFile", &file.path, find_node_span(function.span()), format!("{subject}::{}", function.sig.ident), "Place free functions in a function file, separate from a type family (arch-package.rs.md)".to_owned()));
            }
        }
        findings
    }
}

impl Rule for OneTypeFamilyPerFile {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .crates
            .iter()
            .flat_map(|sources| {
                let mut behaviour = HashMap::new();
                for file in &sources.files {
                    Self::collect_behaviour(
                        &file.syntax.items,
                        &file.module.path,
                        file,
                        &sources.files,
                        &mut behaviour,
                    );
                }
                let no_behaviour = HashSet::new();
                sources
                    .files
                    .iter()
                    .filter(|file| !file.is_generated && !file.module.path.is_empty())
                    .flat_map(|file| {
                        Self::check_file(
                            file,
                            sources,
                            behaviour.get(&file.path).unwrap_or(&no_behaviour),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}
