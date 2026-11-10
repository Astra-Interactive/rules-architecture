use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::check::util::{build_node_finding, format_module_path};
use crate::module::rule::TestAttributeVisitor;
use crate::source::model::{ModuleDeclaration, SourceFile, SourceModule, SourceTree};
use crate::source::util::find_node_span;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{ImplItem, Item, TraitItem};

pub(crate) struct TestMirror;

impl TestMirror {
    fn find_source_path(file: &SourceFile, context: &RuleContext) -> Option<String> {
        let directory = context
            .workspace
            .members
            .iter()
            .find(|member| member.name == file.module.crate_name)?
            .directory
            .as_str();
        let prefix = if directory.is_empty() {
            String::new()
        } else {
            format!("{directory}/")
        };
        let relative = file.path.strip_prefix(&prefix)?;
        let test = relative
            .strip_prefix("tests/it/")
            .or_else(|| relative.strip_prefix("test/"))?;
        Some(format!("{prefix}src/{}.rs", test.strip_suffix("_test.rs")?))
    }

    fn has_behaviour(file: &SourceFile) -> bool {
        file.syntax.items.iter().any(|item| match item {
            Item::Fn(_) => true,
            Item::Impl(item) => item
                .items
                .iter()
                .any(|item| matches!(item, ImplItem::Fn(_))),
            Item::Trait(item) => item
                .items
                .iter()
                .any(|item| matches!(item, TraitItem::Fn(function) if function.default.is_some())),
            _ => false,
        })
    }

    fn is_test_function(function: &syn::ItemFn) -> bool {
        function.attrs.iter().any(|attribute| {
            attribute
                .path()
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "test")
        })
    }

    fn is_test_in_fixture_directory(file: &SourceFile) -> bool {
        let directory = file
            .path
            .rsplit_once('/')
            .map_or("", |(directory, _)| directory);
        directory
            .split('/')
            .any(|segment| segment == "fake" || segment == "memory")
            && file
                .syntax
                .items
                .iter()
                .any(|item| matches!(item, Item::Fn(function) if Self::is_test_function(function)))
    }

    fn check_test_file(
        file: &SourceFile,
        module: &SourceModule,
        declaration: &ModuleDeclaration,
        context: &RuleContext,
        findings: &mut Vec<RuleFinding>,
    ) {
        if file
            .module
            .path
            .iter()
            .any(|segment| segment == "fake" || segment == "memory")
            && !Self::is_test_in_fixture_directory(file)
        {
            return;
        }
        let reason = match file.path.strip_suffix("_test.rs") {
            None => Some("a test file ends in _test.rs"),
            Some(_) => {
                let source = Self::find_source_path(file, context);
                match context
                    .crates
                    .iter()
                    .filter(|candidate| candidate.crate_name == module.id.crate_name)
                    .flat_map(|candidate| &candidate.files)
                    .find(|candidate| source.as_deref() == Some(candidate.path.as_str()))
                {
                    None => Some("the test file has no source file at its mirrored src path"),
                    Some(owner) if !Self::has_behaviour(owner) => {
                        Some("the source file has no behaviour to test")
                    }
                    Some(_) => None,
                }
            }
        };
        let Some(reason) = reason else {
            return;
        };
        findings.push(RuleFinding {
            finding: Finding {
                rule_id: RuleId("TestMirror".to_owned()),
                location: Location {
                    path: declaration.file.clone(),
                    line: Some(declaration.span.start.line),
                },
                subject: format_module_path(&module.id.crate_name, &module.id.path),
                message: reason.to_owned(),
            },
            anchor: Anchor::Module {
                module: module.id.clone(),
                file: declaration.file.clone(),
                body_file: declaration.body_file.clone(),
                declaration: declaration.span,
            },
        });
    }

    fn check_source_file(
        file: &SourceFile,
        tree: Option<SourceTree>,
        findings: &mut Vec<RuleFinding>,
    ) {
        let mut visitor = TestAttributeVisitor {
            file: file.path.clone(),
            subject: format_module_path(&file.module.crate_name, &file.module.path),
            is_root: file.module.path.is_empty(),
            in_test: tree != Some(SourceTree::Src),
            findings: Vec::new(),
        };
        visitor.visit_file(&file.syntax);
        findings.extend(visitor.findings);
        if !file.module.path.is_empty() {
            return;
        }
        for (index, item) in file.syntax.items.iter().enumerate() {
            let is_test_block = matches!(item, Item::Mod(module) if module.ident == "test");
            let has_following_item = index + 1 != file.syntax.items.len();
            if is_test_block && has_following_item {
                findings.push(build_node_finding(
                    "TestMirror",
                    &file.path,
                    find_node_span(item.span()),
                    format_module_path(&file.module.crate_name, &["test".to_owned()]),
                    "the test block is the last item of the crate root".to_owned(),
                ));
            }
        }
    }
}

impl Rule for TestMirror {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for member in &context.workspace.members {
            let directory = if member.directory.is_empty() {
                "tests".to_owned()
            } else {
                format!("{}/tests", member.directory)
            };
            let has_tests = context.workspace.directories.contains(&directory);
            let allowed_it = member.is_proc_macro()
                && !context.workspace.files.iter().any(|file| {
                    file.starts_with(&format!("{directory}/"))
                        && !file.starts_with(&format!("{directory}/it/"))
                })
                && !context.workspace.directories.iter().any(|path| {
                    path.starts_with(&format!("{directory}/"))
                        && path != &format!("{directory}/it")
                        && !path.starts_with(&format!("{directory}/it/"))
                });
            if has_tests && !allowed_it {
                findings.push(RuleFinding {
                    finding: Finding {
                        rule_id: RuleId("TestMirror".to_owned()),
                        location: Location {
                            path: directory,
                            line: None,
                        },
                        subject: format!("{} tests", member.name),
                        message:
                            "tests live in test/; only proc-macro expansion tests use tests/it/"
                                .to_owned(),
                    },
                    anchor: Anchor::Crate(member.name.clone()),
                });
            }
        }
        for sources in &context.crates {
            for file in &sources.files {
                let tree = sources
                    .modules
                    .iter()
                    .find(|module| module.id == file.module)
                    .map(|module| module.tree);
                Self::check_source_file(file, tree, &mut findings);
            }
            for module in sources
                .modules
                .iter()
                .filter(|module| matches!(module.tree, SourceTree::Test | SourceTree::TestsIt))
            {
                let Some(declaration) = &module.declaration else {
                    continue;
                };
                for file in sources.files.iter().filter(|file| file.module == module.id) {
                    Self::check_test_file(file, module, declaration, context, &mut findings);
                }
            }
        }
        findings
    }
}
