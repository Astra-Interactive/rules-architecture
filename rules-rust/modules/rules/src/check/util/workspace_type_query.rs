use crate::check::model::RuleContext;
use crate::source::model::{CfgPredicate, ResolvedTypeSyntax};
use crate::source::model::{ExternalPath, WorkspaceResolvedPath};
use crate::source::syn::WorkspaceTypeResolver;

impl RuleContext {
    pub(crate) fn resolve_outer_workspace_type(
        &self,
        index: usize,
        module: usize,
        typed: &syn::Type,
    ) -> Vec<WorkspaceResolvedPath> {
        WorkspaceTypeResolver::new(self, false)
            .resolve(index, module, typed)
            .items
    }
    pub(crate) fn resolve_canonical_type_path(
        &self,
        index: usize,
        module: usize,
        path: &syn::Path,
    ) -> Vec<WorkspaceResolvedPath> {
        if path.leading_colon.is_none() {
            return self.resolve_workspace_path(index, module, path);
        }
        let segments = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        let Some(first) = segments.first() else {
            return Vec::new();
        };
        let targets = self
            .indexes
            .iter()
            .enumerate()
            .filter(|(_, index)| index.crate_name.replace('-', "_") == *first)
            .flat_map(
                |(index, source)| match source.resolve_segments(0, &segments[1..], 0) {
                    crate::source::model::Resolution::Internal(paths) => paths
                        .into_iter()
                        .map(|path| WorkspaceResolvedPath { index, path })
                        .collect(),
                    _ => Vec::new(),
                },
            )
            .collect::<Vec<_>>();
        if targets.is_empty() {
            self.resolve_workspace_path(index, module, path)
        } else {
            targets
        }
    }

    pub(crate) fn canonical_external_type_path(
        &self,
        index: usize,
        module: usize,
        path: &syn::Path,
    ) -> Option<ExternalPath> {
        let resolution = self.indexes[index].resolve_path(module, path);
        if let crate::source::model::Resolution::External(path) = resolution {
            return Some(self.resolve_external_identity(index, &path));
        }
        let first = path.segments.first()?.ident.to_string();
        if path.leading_colon.is_some() {
            let dependency = self
                .workspace
                .members
                .iter()
                .flat_map(|member| &member.dependencies)
                .find(|dependency| dependency.name.replace('-', "_") == first)?;
            return Some(ExternalPath {
                package: dependency.name.clone(),
                segments: path
                    .segments
                    .iter()
                    .skip(1)
                    .map(|segment| segment.ident.to_string())
                    .collect(),
                member: self
                    .workspace
                    .members
                    .iter()
                    .position(|member| member.name == dependency.name),
            });
        }
        if !matches!(resolution, crate::source::model::Resolution::Local)
            || path.segments.len() != 1
        {
            return None;
        }
        let canonical = match first.as_str() {
            "Box" => "alloc::boxed::Box",
            "Vec" => "alloc::vec::Vec",
            "String" => "alloc::string::String",
            "Option" => "core::option::Option",
            "Result" => "core::result::Result",
            "Send" => "core::marker::Send",
            "Sync" => "core::marker::Sync",
            "Fn" => "core::ops::Fn",
            "FnMut" => "core::ops::FnMut",
            "FnOnce" => "core::ops::FnOnce",
            _ => return None,
        };
        Some(self.resolve_external_identity(index, canonical))
    }
    pub(crate) fn resolve_external_type_paths_under(
        &self,
        index: usize,
        module: usize,
        typed: &syn::Type,
        cfg: &CfgPredicate,
    ) -> Vec<ExternalPath> {
        WorkspaceTypeResolver::new(self, true)
            .resolve_under(index, module, typed, cfg)
            .external
    }

    pub(crate) fn resolve_outer_external_type_paths_under(
        &self,
        index: usize,
        module: usize,
        typed: &syn::Type,
        cfg: &CfgPredicate,
    ) -> Vec<ExternalPath> {
        WorkspaceTypeResolver::new(self, false)
            .resolve_under(index, module, typed, cfg)
            .external
    }
    pub(crate) fn resolve_external_call_paths(
        &self,
        index: usize,
        module: usize,
        path: &syn::Path,
    ) -> Vec<ExternalPath> {
        if let crate::source::model::Resolution::External(path) =
            self.indexes[index].resolve_path(module, path)
        {
            return vec![self.resolve_external_identity(index, &path)];
        }
        let mut owner = path.clone();
        let Some(member) = owner.segments.pop() else {
            return Vec::new();
        };
        if owner.segments.is_empty() {
            return Vec::new();
        }
        let typed = syn::Type::Path(syn::TypePath {
            attrs: Vec::new(),
            qself: None,
            path: owner,
        });
        self.resolve_outer_external_type_paths(index, module, &typed)
            .into_iter()
            .map(|mut path| {
                path.segments.push(member.ident.to_string());
                path
            })
            .collect()
    }

    fn resolve_type_syntax_at(
        &self,
        index: usize,
        module: usize,
        typed: &syn::Type,
        depth: usize,
        cfg: CfgPredicate,
        incomplete: bool,
    ) -> Vec<ResolvedTypeSyntax> {
        if depth > 8 || !cfg.is_satisfiable() {
            return Vec::new();
        }
        let inner = match typed {
            syn::Type::Paren(paren) => Some(&*paren.elem),
            syn::Type::Group(group) => Some(&*group.elem),
            _ => None,
        };
        if let Some(inner) = inner {
            return self.resolve_type_syntax_at(index, module, inner, depth, cfg, incomplete);
        }
        if let syn::Type::Path(path) = typed {
            let aliases: Vec<_> = self
                .resolve_canonical_type_path(index, module, &path.path)
                .into_iter()
                .filter_map(|target| {
                    let item = target.path.item?;
                    let declaration = &self.indexes[target.index].items[item];
                    let syn::Item::Type(alias) = &declaration.syntax else {
                        return None;
                    };
                    Some((target, declaration, alias))
                })
                .collect();
            if !aliases.is_empty() {
                return aliases
                    .into_iter()
                    .flat_map(|(target, declaration, alias)| {
                        let mut folder = crate::source::syn::AliasSyntaxFolder::new(
                            self,
                            target.index,
                            declaration.module,
                        );
                        folder.cfg = cfg
                            .in_scope(index)
                            .and(&target.path.cfg.in_scope(target.index));
                        folder.bind(alias, &path.path, index, module);
                        let expanded = syn::fold::Fold::fold_type(&mut folder, (*alias.ty).clone());
                        self.resolve_type_syntax_at(
                            target.index,
                            declaration.module,
                            &expanded,
                            depth + 1,
                            cfg.in_scope(index)
                                .and(&target.path.cfg.in_scope(target.index)),
                            incomplete || folder.incomplete,
                        )
                    })
                    .collect();
            }
        }
        vec![ResolvedTypeSyntax {
            index,
            module,
            typed: typed.clone(),
            cfg,
            has_unsubstituted_parameters: incomplete,
        }]
    }

    pub(crate) fn resolve_type_syntax(
        &self,
        index: usize,
        module: usize,
        typed: &syn::Type,
    ) -> Vec<ResolvedTypeSyntax> {
        self.resolve_type_syntax_under(index, module, typed, &CfgPredicate::All(Vec::new()))
    }

    pub(crate) fn resolve_type_syntax_under(
        &self,
        index: usize,
        module: usize,
        typed: &syn::Type,
        cfg: &CfgPredicate,
    ) -> Vec<ResolvedTypeSyntax> {
        self.resolve_type_syntax_at(index, module, typed, 0, cfg.clone(), false)
    }

    pub(crate) fn resolve_workspace_type(
        &self,
        index: usize,
        module: usize,
        typed: &syn::Type,
    ) -> Vec<WorkspaceResolvedPath> {
        WorkspaceTypeResolver::new(self, true)
            .resolve(index, module, typed)
            .items
    }

    pub(crate) fn resolve_external_type_paths(
        &self,
        index: usize,
        module: usize,
        typed: &syn::Type,
    ) -> Vec<ExternalPath> {
        WorkspaceTypeResolver::new(self, true)
            .resolve(index, module, typed)
            .external
    }

    pub(crate) fn resolve_outer_external_type_paths(
        &self,
        index: usize,
        module: usize,
        typed: &syn::Type,
    ) -> Vec<ExternalPath> {
        WorkspaceTypeResolver::new(self, false)
            .resolve(index, module, typed)
            .external
    }
}
