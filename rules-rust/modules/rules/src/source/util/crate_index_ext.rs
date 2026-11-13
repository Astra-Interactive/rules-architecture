use crate::source::model::CrateIndex;
use crate::source::model::IndexedModule;
use crate::source::model::Resolution;
use crate::source::model::ResolvedPath;
use syn::ext::IdentExt;

impl CrateIndex {
    fn resolve_type_items_at(
        &self,
        module: usize,
        typed: &syn::Type,
        depth: usize,
        cfg: &crate::source::model::CfgPredicate,
    ) -> Vec<usize> {
        if depth > 8 {
            return Vec::new();
        }
        let path = match typed {
            syn::Type::Path(path) => &path.path,
            syn::Type::Reference(reference) => {
                return self.resolve_type_items_at(module, &reference.elem, depth, cfg);
            }
            syn::Type::Ptr(pointer) => {
                return self.resolve_type_items_at(module, &pointer.elem, depth, cfg);
            }
            syn::Type::Paren(paren) => {
                return self.resolve_type_items_at(module, &paren.elem, depth, cfg);
            }
            syn::Type::Group(group) => {
                return self.resolve_type_items_at(module, &group.elem, depth, cfg);
            }
            _ => return Vec::new(),
        };
        let Resolution::Internal(targets) = self.resolve_path(module, path) else {
            return Vec::new();
        };
        targets
            .into_iter()
            .flat_map(|target| {
                let Some(identity) = target.item else {
                    return Vec::new();
                };
                let cfg = cfg.and(&target.cfg);
                if !cfg.is_satisfiable() {
                    return Vec::new();
                }
                let item = &self.items[identity];
                match &item.syntax {
                    syn::Item::Type(alias) => {
                        self.resolve_type_items_at(item.module, &alias.ty, depth + 1, &cfg)
                    }
                    _ => vec![identity],
                }
            })
            .collect()
    }

    pub(crate) fn resolve_type_items(&self, module: usize, typed: &syn::Type) -> Vec<usize> {
        self.resolve_type_items_at(
            module,
            typed,
            0,
            &crate::source::model::CfgPredicate::All(Vec::new()),
        )
    }
    pub(crate) fn find_module(&self, id: usize) -> Option<&IndexedModule> {
        self.modules.get(id)
    }

    pub(crate) fn resolve_path(&self, module: usize, path: &syn::Path) -> Resolution {
        let mut segments: Vec<String> = path
            .segments
            .iter()
            .map(|segment| segment.ident.unraw().to_string())
            .collect();
        if path.leading_colon.is_some() {
            segments.insert(0, String::new());
        }
        self.resolve_segments(module, &segments, 0)
    }

    pub(crate) fn resolve_segments(
        &self,
        module: usize,
        path: &[String],
        depth: usize,
    ) -> Resolution {
        let Some(first) = path.first() else {
            return Resolution::Internal(vec![ResolvedPath {
                module,
                item: None,
                cfg: self.modules[module].cfg.clone(),
            }]);
        };
        if depth > 8 {
            return Resolution::Unresolved;
        }
        match first.as_str() {
            "" => {
                let Some(name) = path.get(1) else {
                    return Resolution::Unresolved;
                };
                if self.dependencies.contains(name)
                    || ["std", "core", "alloc", "proc_macro", "test"].contains(&name.as_str())
                {
                    return Resolution::External(path[1..].join("::"));
                }
                return Resolution::Unresolved;
            }
            "crate" => return self.resolve_member(0, &path[1..], depth),
            "self" => return self.resolve_member(module, &path[1..], depth),
            "super" => {
                return self.modules[module]
                    .parent
                    .map_or(Resolution::Unresolved, |parent| {
                        self.resolve_segments(parent, &path[1..], depth)
                    });
            }
            _ => {}
        }
        let local = self.resolve_member(module, path, depth);
        if local != Resolution::Unresolved {
            return local;
        }
        if self
            .dependencies
            .iter()
            .any(|dependency| dependency == first)
            || matches!(
                first.as_str(),
                "std" | "core" | "alloc" | "proc_macro" | "test"
            )
        {
            return Resolution::External(path.join("::"));
        }
        if let Some(resolved) = Self::resolve_prelude(path) {
            return resolved;
        }
        if Self::is_builtin(first) {
            return Resolution::Local;
        }
        self.resolve_globs(module, path, depth)
    }

    fn resolve_prelude(path: &[String]) -> Option<Resolution> {
        if let Some(canonical) = match path.first()?.as_str() {
            "drop" => Some("core::mem::drop"),
            "Drop" => Some("core::ops::Drop"),
            "size_of" => Some("core::mem::size_of"),
            "size_of_val" => Some("core::mem::size_of_val"),
            "align_of" => Some("core::mem::align_of"),
            "align_of_val" => Some("core::mem::align_of_val"),
            _ => None,
        } {
            let suffix = path.iter().skip(1).cloned().collect::<Vec<_>>().join("::");
            return Some(Resolution::External(if suffix.is_empty() {
                canonical.to_owned()
            } else {
                format!("{canonical}::{suffix}")
            }));
        }
        None
    }

    fn is_builtin(name: &str) -> bool {
        Self::BUILTIN_NAMES.contains(&name)
    }

    const BUILTIN_NAMES: &[&str] = &[
        "Self",
        "bool",
        "char",
        "str",
        "u8",
        "u16",
        "u32",
        "u64",
        "u128",
        "usize",
        "i8",
        "i16",
        "i32",
        "i64",
        "i128",
        "isize",
        "f16",
        "f32",
        "f64",
        "f128",
        "String",
        "Vec",
        "Box",
        "Option",
        "Result",
        "Some",
        "None",
        "Ok",
        "Err",
        "Clone",
        "Copy",
        "Send",
        "Sync",
        "Sized",
        "Unpin",
        "Drop",
        "Default",
        "From",
        "Into",
        "TryFrom",
        "TryInto",
        "Iterator",
        "IntoIterator",
        "DoubleEndedIterator",
        "ExactSizeIterator",
        "Extend",
        "FromIterator",
        "AsRef",
        "AsMut",
        "ToOwned",
        "ToString",
        "Fn",
        "FnMut",
        "FnOnce",
        "Eq",
        "PartialEq",
        "Ord",
        "PartialOrd",
        "Debug",
        "Future",
    ];

    fn resolve_member(&self, module: usize, path: &[String], depth: usize) -> Resolution {
        let Some(first) = path.first() else {
            return Resolution::Internal(vec![ResolvedPath {
                module,
                item: None,
                cfg: self.modules[module].cfg.clone(),
            }]);
        };
        let children: Vec<usize> = self
            .modules
            .iter()
            .filter(|child| child.parent == Some(module) && child.module.path.last() == Some(first))
            .map(|child| child.id)
            .collect();
        if !children.is_empty() {
            return Self::combine(
                children
                    .into_iter()
                    .map(|child| self.resolve_member(child, &path[1..], depth)),
            );
        }
        let items: Vec<ResolvedPath> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.module == module && item.name == *first)
            .map(|(item, _)| ResolvedPath {
                module,
                item: Some(item),
                cfg: self.items[item].cfg.clone(),
            })
            .collect();
        if !items.is_empty() {
            return Resolution::Internal(items);
        }
        let imports: Vec<_> = self
            .imports
            .iter()
            .filter(|binding| {
                binding.module == module && !binding.is_glob && binding.name.as_ref() == Some(first)
            })
            .collect();
        if imports.is_empty() {
            return self.resolve_globs(module, path, depth);
        }
        Self::combine(imports.into_iter().map(|binding| {
            let mut target = binding.path.clone();
            target.extend_from_slice(&path[1..]);
            Self::with_predicate(
                self.resolve_segments(module, &target, depth + 1),
                &binding.cfg,
            )
        }))
    }

    fn resolve_globs(&self, module: usize, path: &[String], depth: usize) -> Resolution {
        let mut results = Vec::new();
        let mut external = false;
        for binding in self
            .imports
            .iter()
            .filter(|binding| binding.module == module && binding.is_glob)
        {
            match self.resolve_segments(module, &binding.path, depth + 1) {
                Resolution::Internal(targets) => {
                    results.extend(targets.into_iter().map(|target| {
                        Self::with_predicate(
                            self.resolve_member(target.module, path, depth + 1),
                            &binding.cfg.and(&target.cfg),
                        )
                    }));
                }
                Resolution::External(_) | Resolution::ExternalGlob => external = true,
                _ => {}
            }
        }
        let resolved = Self::combine(results);
        if resolved == Resolution::Unresolved && external {
            return Resolution::ExternalGlob;
        }
        resolved
    }

    fn combine(results: impl IntoIterator<Item = Resolution>) -> Resolution {
        let mut internal = Vec::new();
        let mut external = None;
        for result in results {
            match result {
                Resolution::Internal(targets) => internal.extend(targets),
                Resolution::External(path) => external = Some(path),
                _ => {}
            }
        }
        internal.sort_by_key(|target| (target.module, target.item));
        internal.dedup();
        if !internal.is_empty() {
            return Resolution::Internal(internal);
        }
        external.map_or(Resolution::Unresolved, Resolution::External)
    }

    fn with_predicate(
        resolution: Resolution,
        predicate: &crate::source::model::CfgPredicate,
    ) -> Resolution {
        match resolution {
            Resolution::Internal(targets) => Resolution::Internal(
                targets
                    .into_iter()
                    .map(|target| ResolvedPath {
                        cfg: target.cfg.and(predicate),
                        ..target
                    })
                    .collect(),
            ),
            other => other,
        }
    }
}
