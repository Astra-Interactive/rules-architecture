use crate::source::model::CrateSources;
use crate::source::model::Expectation;
use crate::source::model::ExpectationPosition;
use crate::source::model::ModuleDeclaration;
use crate::source::model::ModuleFrame;
use crate::source::model::ModuleId;
use crate::source::model::SourceFile;
use crate::source::model::SourceLoadError;
use crate::source::model::SourceModule;
use crate::source::model::SourceTree;
use crate::source::model::UnanalysableSite;
use crate::source::model::UnanalysableSource;
use crate::source::syn::ExpectationVisitor;
use crate::source::util::find_declared_visibility;
use crate::source::util::find_node_span;
use crate::source::util::find_path_attribute;
use crate::source::util::find_single_target_value;
use crate::source::util::has_conditional_path;
use crate::source::util::scan_comments;
use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use std::fs;
use std::ops::Range;
use std::path::PathBuf;
use syn::Item;
use syn::ItemMod;
use syn::ext::IdentExt;
use syn::spanned::Spanned;

pub(crate) struct ModuleTreeWalker {
    root: PathBuf,
    crate_name: String,
    target: String,
    modules: Vec<SourceModule>,
    files: Vec<SourceFile>,
    expectations: Vec<Expectation>,
    unanalysable: Vec<UnanalysableSource>,
}

impl ModuleTreeWalker {
    pub(crate) fn new(root: PathBuf, crate_name: String, target: String) -> Self {
        Self {
            root,
            crate_name,
            target,
            modules: Vec::new(),
            files: Vec::new(),
            expectations: Vec::new(),
            unanalysable: Vec::new(),
        }
    }

    fn normalize_path(path: &str) -> String {
        let mut segments: Vec<&str> = Vec::new();
        for segment in path.split('/') {
            match segment {
                "" | "." => {}
                ".." => {
                    segments.pop();
                }
                other => segments.push(other),
            }
        }
        segments.join("/")
    }

    fn find_child_directory(file: &str, is_root_file: bool) -> String {
        let parent = file
            .rsplit_once('/')
            .map_or_else(String::new, |(directory, _)| directory.to_owned());
        let stem = file
            .rsplit('/')
            .next()
            .unwrap_or(file)
            .trim_end_matches(".rs");
        if is_root_file || stem == "mod" {
            return parent;
        }
        format!("{parent}/{stem}")
    }

    fn collect_token_ranges(stream: TokenStream, ranges: &mut Vec<Range<usize>>) {
        for token in stream {
            match token {
                TokenTree::Group(group) => {
                    ranges.push(group.span_open().byte_range());
                    ranges.push(group.span_close().byte_range());
                    Self::collect_token_ranges(group.stream(), ranges);
                }
                other => ranges.push(other.span().byte_range()),
            }
        }
    }

    fn is_generated(text: &str) -> bool {
        text.lines().next().is_some_and(|line| {
            let trimmed = line.trim_start();
            let is_comment = trimmed.starts_with("//") || trimmed.starts_with("/*");
            is_comment && trimmed.contains("@generated")
        })
    }

    fn build_module_id(&self, path: &[String]) -> ModuleId {
        ModuleId {
            crate_name: self.crate_name.clone(),
            target: self.target.clone(),
            path: path.to_vec(),
        }
    }

    fn load_file(
        &mut self,
        relative: &str,
        module_path: &[String],
        is_file_module: bool,
    ) -> Result<Vec<Item>, SourceLoadError> {
        let module = self.build_module_id(module_path);
        if let Some(file) = self
            .files
            .iter()
            .find(|file| file.path == relative && file.module == module)
        {
            return Ok(file.syntax.items.clone());
        }
        let previously_loaded = self.files.iter().any(|file| file.path == relative);
        let text = fs::read_to_string(self.root.join(relative)).map_err(|error| {
            SourceLoadError::Read {
                path: relative.to_owned(),
                source: error.into(),
            }
        })?;
        let syntax = syn::parse_file(&text).map_err(|error| SourceLoadError::Parse {
            path: relative.to_owned(),
            source: error.into(),
        })?;
        let mut ranges = Vec::new();
        if let Ok(stream) = text.parse::<TokenStream>() {
            Self::collect_token_ranges(stream, &mut ranges);
        }
        let visitor = ExpectationVisitor::new(relative.to_owned(), module.clone());
        self.expectations.extend(
            visitor
                .collect_file(&syntax, is_file_module)
                .into_iter()
                .filter(|expectation| {
                    !previously_loaded
                        || matches!(
                            expectation.position,
                            ExpectationPosition::ModuleInner(_)
                                | ExpectationPosition::InlineModuleInner { .. }
                                | ExpectationPosition::ModuleDeclaration { .. }
                        )
                }),
        );
        let items = syntax.items.clone();
        self.files.push(SourceFile {
            path: relative.to_owned(),
            module,
            is_generated: Self::is_generated(&text),
            comments: scan_comments(&text, &ranges),
            syntax,
        });
        Ok(items)
    }

    fn report_node(&mut self, file: &str, item: &impl Spanned, subject: String, message: String) {
        self.unanalysable.push(UnanalysableSource {
            site: UnanalysableSite::Node {
                file: file.to_owned(),
                span: find_node_span(item.span()),
            },
            subject,
            message,
        });
    }

    fn format_item_path(&self, frame: &ModuleFrame, name: &str) -> String {
        let mut segments = vec![self.crate_name.replace('-', "_")];
        segments.extend(frame.path.iter().cloned());
        segments.push(name.to_owned());
        segments.join("::")
    }

    fn check_macro(&mut self, item: &syn::ItemMacro, frame: &ModuleFrame) {
        let name = item
            .mac
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            .unwrap_or_default();
        if name == "macro_rules" {
            return;
        }
        let subject = self.format_item_path(frame, &format!("{name}!"));
        if name == "include" {
            let message = "`include!` at item position hides the items it loads".to_owned();
            self.report_node(&frame.file, item, subject, message);
            return;
        }
        if frame.is_root_file {
            let message = format!(
                "the item macro `{name}!` in the root file may generate items the linter cannot see"
            );
            self.report_node(&frame.file, item, subject, message);
        }
    }

    fn select_child_tree(frame_tree: SourceTree, is_crate_root: bool, name: &str) -> SourceTree {
        match (is_crate_root, frame_tree, name) {
            (true, SourceTree::Src, "test") => SourceTree::Test,
            (true, SourceTree::Src, "fixture") => SourceTree::Fixture,
            _ => frame_tree,
        }
    }

    fn resolve_module_file(
        &self,
        directory: &str,
        name: &str,
        path_attribute: Option<&str>,
    ) -> Option<String> {
        let candidates = match path_attribute {
            Some(path) => vec![format!("{directory}/{path}")],
            None => vec![
                format!("{directory}/{name}.rs"),
                format!("{directory}/{name}/mod.rs"),
            ],
        };
        candidates
            .into_iter()
            .map(|candidate| Self::normalize_path(&candidate))
            .find(|candidate| self.root.join(candidate).is_file())
    }

    fn push_module(&mut self, item: &ItemMod, child: &ModuleFrame, declaring_file: &str) {
        let use_visibilities = item
            .content
            .iter()
            .flat_map(|(_, items)| items)
            .filter_map(|inner| match inner {
                Item::Use(use_item) => Some(find_declared_visibility(&use_item.vis)),
                _ => None,
            })
            .collect();
        self.modules.push(SourceModule {
            id: self.build_module_id(&child.path),
            declaration: Some(ModuleDeclaration {
                file: declaring_file.to_owned(),
                body_file: child.file.clone(),
                span: find_node_span(item.span()),
                is_raw_identifier: item.ident.to_string().starts_with("r#"),
                is_inline: item.content.is_some(),
                visibility: find_declared_visibility(&item.vis),
                use_visibilities,
                target_value: find_single_target_value(&item.attrs),
            }),
            tree: child.tree,
        });
    }

    fn walk_file_module(
        &mut self,
        item: &ItemMod,
        frame: &ModuleFrame,
        child: ModuleFrame,
        path_attribute: Option<&str>,
    ) {
        let name = item.ident.unraw().to_string();
        let subject = self.format_item_path(frame, &name);
        let Some(file) = self.resolve_module_file(&frame.directory, &name, path_attribute) else {
            let message = format!("no file for `mod {name};` under {}", frame.directory);
            self.report_node(&frame.file, item, subject, message);
            return;
        };
        let items = match self.load_file(&file, &child.path, true) {
            Ok(items) => items,
            Err(error) => {
                self.report_node(&frame.file, item, subject, error.to_string());
                return;
            }
        };
        let child = ModuleFrame {
            directory: Self::find_child_directory(&file, path_attribute.is_some()),
            file,
            is_root_file: false,
            ..child
        };
        self.push_module(item, &child, &frame.file);
        self.walk_items(&items, &child);
    }

    fn walk_module(&mut self, item: &ItemMod, frame: &ModuleFrame) {
        let name = item.ident.unraw().to_string();
        if has_conditional_path(&item.attrs) {
            let subject = self.format_item_path(frame, &name);
            let message = "`cfg_attr(…, path = …)` hides which file the module loads".to_owned();
            self.report_node(&frame.file, item, subject, message);
            return;
        }
        let is_crate_root = frame.path.is_empty() && frame.is_root_file;
        let tree = Self::select_child_tree(frame.tree, is_crate_root, &name);
        let path_attribute = find_path_attribute(&item.attrs);
        let mut path = frame.path.clone();
        path.push(name.clone());
        let Some((_, items)) = &item.content else {
            let child = ModuleFrame {
                path,
                file: String::new(),
                directory: String::new(),
                tree,
                is_root_file: false,
            };
            self.walk_file_module(item, frame, child, path_attribute.as_deref());
            return;
        };
        let directory = match &path_attribute {
            Some(relative) => Self::normalize_path(&format!("{}/{relative}", frame.directory)),
            None => format!("{}/{name}", frame.directory),
        };
        let child = ModuleFrame {
            path,
            file: frame.file.clone(),
            directory,
            tree,
            is_root_file: frame.is_root_file,
        };
        self.push_module(item, &child, &frame.file);
        self.walk_items(items, &child);
    }

    fn walk_items(&mut self, items: &[Item], frame: &ModuleFrame) {
        for item in items {
            match item {
                Item::Mod(module) => self.walk_module(module, frame),
                Item::Macro(macro_item) => self.check_macro(macro_item, frame),
                _ => {}
            }
        }
    }

    pub(crate) fn walk_crate(mut self, root_file: &str, tree: SourceTree) -> CrateSources {
        match self.load_file(root_file, &[], false) {
            Ok(items) => {
                self.modules.push(SourceModule {
                    id: self.build_module_id(&[]),
                    declaration: None,
                    tree,
                });
                let frame = ModuleFrame {
                    path: Vec::new(),
                    file: root_file.to_owned(),
                    directory: Self::find_child_directory(root_file, true),
                    tree,
                    is_root_file: true,
                };
                self.walk_items(&items, &frame);
            }
            Err(error) => self.unanalysable.push(UnanalysableSource {
                site: UnanalysableSite::Crate {
                    path: root_file.to_owned(),
                },
                subject: format!("{} {root_file}", self.crate_name),
                message: error.to_string(),
            }),
        }
        CrateSources {
            crate_name: self.crate_name,
            target: self.target,
            modules: self.modules,
            files: self.files,
            expectations: self.expectations,
            unanalysable: self.unanalysable,
            undeclared_files: Vec::new(),
        }
    }
}
