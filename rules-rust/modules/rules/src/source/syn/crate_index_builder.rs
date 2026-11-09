use crate::source::model::CfgPredicate;
use crate::source::model::CrateIndex;
use crate::source::model::CrateSources;
use crate::source::model::ImportBinding;
use crate::source::model::IndexedImpl;
use crate::source::model::IndexedItem;
use crate::source::model::IndexedModule;
use crate::source::model::ModuleId;
use crate::source::model::SourceTree;
use crate::source::syn::PathEdgeVisitor;
use crate::source::util::find_node_span;
use crate::workspace::model::Workspace;
use syn::Item;
use syn::UseTree;
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct CrateIndexBuilder<'a> {
    sources: &'a CrateSources,
    index: CrateIndex,
}

impl<'a> CrateIndexBuilder<'a> {
    pub(crate) fn build_all(sources: &'a [CrateSources], workspace: &Workspace) -> Vec<CrateIndex> {
        sources
            .iter()
            .map(|sources| {
                let mut dependencies = workspace
                    .members
                    .iter()
                    .find(|member| member.name == sources.crate_name)
                    .map_or_else(Vec::new, |member| {
                        member
                            .dependencies
                            .iter()
                            .map(|dependency| dependency.code_name.clone())
                            .collect()
                    });
                dependencies.extend(
                    workspace
                        .members
                        .iter()
                        .filter(|member| member.name == sources.crate_name)
                        .flat_map(|member| {
                            member
                                .targets
                                .iter()
                                .filter(|target| {
                                    target
                                        .kinds
                                        .iter()
                                        .any(|kind| kind == "lib" || kind == "rlib")
                                })
                                .map(|target| target.name.replace('-', "_"))
                        }),
                );
                let mut builder = Self::new(sources, dependencies);
                if let Some(member) = workspace
                    .members
                    .iter()
                    .find(|member| member.name == sources.crate_name)
                {
                    builder.index.dependency_packages = member
                        .dependencies
                        .iter()
                        .map(|dependency| {
                            (
                                dependency.code_name.clone(),
                                dependency.name.replace('-', "_"),
                            )
                        })
                        .collect();
                }
                builder.build()
            })
            .collect()
    }

    pub(crate) fn new(sources: &'a CrateSources, dependencies: Vec<String>) -> Self {
        Self {
            sources,
            index: CrateIndex {
                crate_name: sources.crate_name.clone(),
                target: sources.target.clone(),
                dependencies,
                dependency_packages: std::collections::BTreeMap::new(),
                modules: Vec::new(),
                items: Vec::new(),
                implementations: Vec::new(),
                imports: Vec::new(),
                edges: Vec::new(),
                unanalysable: Vec::new(),
                references: Vec::new(),
            },
        }
    }

    pub(crate) fn build(mut self) -> CrateIndex {
        if let Some(file) = self
            .sources
            .files
            .iter()
            .find(|file| file.module.path.is_empty())
        {
            let tree = self
                .sources
                .modules
                .first()
                .map_or(SourceTree::Src, |module| module.tree);
            self.index.modules.push(IndexedModule {
                id: 0,
                parent: None,
                module: file.module.clone(),
                file: file.path.clone(),
                span: find_node_span(file.syntax.span()),
                tree,
                cfg: CfgPredicate::from_attributes(&file.syntax.attrs),
                is_file_module: false,
            });
            self.visit_items(&file.syntax.items, 0);
        }
        self.collect_edges();
        self.index
    }

    fn collect_edges(&mut self) {
        let mut edges = Vec::new();
        let mut unanalysable = Vec::new();
        let mut references = Vec::new();
        for item in &self.index.items {
            let mut visitor =
                PathEdgeVisitor::new(&self.index, item.module, item.span, item.cfg.clone());
            visitor.visit_item(&item.syntax);
            edges.extend(visitor.edges);
            unanalysable.extend(visitor.unanalysable);
            references.extend(visitor.references);
        }
        for implementation in &self.index.implementations {
            let mut visitor = PathEdgeVisitor::new(
                &self.index,
                implementation.module,
                implementation.span,
                implementation.cfg.clone(),
            );
            visitor.visit_item_impl(&implementation.syntax);
            edges.extend(visitor.edges);
            unanalysable.extend(visitor.unanalysable);
            references.extend(visitor.references);
        }
        for import in &self.index.imports {
            let text = import.path.join("::");
            let Ok(path) = syn::parse_str::<syn::Path>(&text) else {
                continue;
            };
            let mut visitor =
                PathEdgeVisitor::new(&self.index, import.module, import.span, import.cfg.clone());
            visitor.record_resolution(&path);
            for edge in &mut visitor.edges {
                edge.span = import.span;
            }
            for reference in &mut visitor.references {
                reference.span = import.span;
            }
            edges.extend(visitor.edges);
            unanalysable.extend(visitor.unanalysable);
            references.extend(visitor.references);
        }
        self.index.edges = edges;
        self.index.unanalysable = unanalysable;
        self.index.references = references;
    }

    fn visit_items(&mut self, items: &[Item], module: usize) {
        for item in items {
            match item {
                Item::Mod(child) => self.visit_module(child, module),
                Item::Use(import) => self.visit_import(
                    &import.tree,
                    if import.leading_colon.is_some() {
                        vec![String::new()]
                    } else {
                        Vec::new()
                    },
                    module,
                    import,
                ),
                Item::Impl(block) => self.index.implementations.push(IndexedImpl {
                    module,
                    file: self.index.modules[module].file.clone(),
                    span: find_node_span(block.span()),
                    cfg: self.index.modules[module]
                        .cfg
                        .and(&CfgPredicate::from_attributes(&block.attrs)),
                    syntax: block.clone(),
                }),
                other => self.visit_declaration(other, module),
            }
        }
    }

    fn visit_declaration(&mut self, item: &Item, module: usize) {
        let declaration = match item {
            Item::Struct(item) => Some((&item.ident, &item.attrs)),
            Item::Enum(item) => Some((&item.ident, &item.attrs)),
            Item::Union(item) => Some((&item.ident, &item.attrs)),
            Item::Trait(item) => Some((&item.ident, &item.attrs)),
            Item::TraitAlias(item) => Some((&item.ident, &item.attrs)),
            Item::Type(item) => Some((&item.ident, &item.attrs)),
            Item::Fn(item) => Some((&item.sig.ident, &item.attrs)),
            Item::Const(item) => Some((&item.ident, &item.attrs)),
            Item::Static(item) => Some((&item.ident, &item.attrs)),
            Item::Macro(item) => item.ident.as_ref().map(|name| (name, &item.attrs)),
            _ => None,
        };
        let Some((name, attrs)) = declaration else {
            return;
        };
        let parent = &self.index.modules[module];
        self.index.items.push(IndexedItem {
            module,
            file: parent.file.clone(),
            span: find_node_span(item.span()),
            name: name.unraw().to_string(),
            cfg: parent.cfg.and(&CfgPredicate::from_attributes(attrs)),
            syntax: item.clone(),
        });
    }

    fn visit_module(&mut self, item: &syn::ItemMod, parent: usize) {
        let ancestor = &self.index.modules[parent];
        let mut path = ancestor.module.path.clone();
        path.push(item.ident.unraw().to_string());
        let declaration = self.sources.modules.iter().find(|candidate| {
            candidate.id.path == path
                && candidate.declaration.as_ref().is_some_and(|declaration| {
                    declaration.file == ancestor.file
                        && declaration.span == find_node_span(item.span())
                })
        });
        let Some(declaration) = declaration else {
            return;
        };
        let Some(origin) = declaration.declaration.as_ref() else {
            return;
        };
        let file = origin.body_file.clone();
        let mut cfg = ancestor
            .cfg
            .and(&CfgPredicate::from_attributes(&item.attrs));
        let items = match &item.content {
            Some((_, items)) => items.clone(),
            None => {
                let Some(source) = self
                    .sources
                    .files
                    .iter()
                    .find(|source| source.path == file && source.module.path == path)
                else {
                    return;
                };
                cfg = cfg.and(&CfgPredicate::from_attributes(&source.syntax.attrs));
                source.syntax.items.clone()
            }
        };
        let id = self.index.modules.len();
        self.index.modules.push(IndexedModule {
            id,
            parent: Some(parent),
            module: ModuleId {
                path,
                ..ancestor.module.clone()
            },
            file,
            span: origin.span,
            tree: declaration.tree,
            cfg,
            is_file_module: item.content.is_none(),
        });
        self.visit_items(&items, id);
    }

    fn visit_import(
        &mut self,
        tree: &UseTree,
        prefix: Vec<String>,
        module: usize,
        item: &syn::ItemUse,
    ) {
        match tree {
            UseTree::Path(path) => {
                let mut next = prefix;
                next.push(path.ident.unraw().to_string());
                self.visit_import(&path.tree, next, module, item);
            }
            UseTree::Group(group) => {
                for child in &group.items {
                    self.visit_import(child, prefix.clone(), module, item);
                }
            }
            other => {
                let mut path = prefix;
                let (name, is_glob) = match other {
                    UseTree::Name(name) if name.ident == "self" => (path.last().cloned(), false),
                    UseTree::Name(name) => {
                        path.push(name.ident.unraw().to_string());
                        (Some(name.ident.unraw().to_string()), false)
                    }
                    UseTree::Rename(rename) => {
                        if rename.ident != "self" {
                            path.push(rename.ident.unraw().to_string());
                        }
                        (Some(rename.rename.unraw().to_string()), false)
                    }
                    _ => (None, true),
                };
                let parent = &self.index.modules[module];
                self.index.imports.push(ImportBinding {
                    module,
                    file: parent.file.clone(),
                    span: find_node_span(item.span()),
                    name,
                    path,
                    is_glob,
                    is_renamed: matches!(other, UseTree::Rename(_)),
                    is_public: !matches!(item.vis, syn::Visibility::Inherited),
                    is_externally_public: matches!(item.vis, syn::Visibility::Public(_)),
                    is_crate_public: matches!(&item.vis,syn::Visibility::Restricted(restriction) if restriction.path.is_ident("crate")),
                    cfg: parent.cfg.and(&CfgPredicate::from_attributes(&item.attrs)),
                });
            }
        }
    }
}
