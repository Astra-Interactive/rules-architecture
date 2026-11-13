use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::check::util::{build_node_finding, format_module_path};
use crate::config::model::Role;
use crate::source::model::SourceFile;
use crate::source::util::{find_node_span, format_tokens};
use crate::workspace::model::{Member, Target};
use syn::spanned::Spanned;
use syn::{Item, ItemMod, Visibility};

pub(crate) struct RootFileShape;

impl RootFileShape {
    fn report(file: &SourceFile, item: &Item, reason: &str, findings: &mut Vec<RuleFinding>) {
        findings.push(build_node_finding(
            "RootFileShape",
            &file.path,
            find_node_span(item.span()),
            format_module_path(&file.module.crate_name, &file.module.path),
            reason.to_owned(),
        ));
    }

    fn check_block(file: &SourceFile, module: &ItemMod, findings: &mut Vec<RuleFinding>) {
        let Some((_, items)) = &module.content else {
            return;
        };
        let mut saw_module = false;
        for item in items {
            match item {
                Item::Use(_) if saw_module => Self::report(
                    file,
                    item,
                    "use lines precede mod lines inside a module block",
                    findings,
                ),
                Item::Use(_) => {}
                Item::Mod(child) => {
                    saw_module = true;
                    Self::check_block(file, child, findings);
                }
                Item::Macro(call) if call.ident.is_none() => {}
                _ => Self::report(
                    file,
                    item,
                    "a module block in the root file holds only use and mod items",
                    findings,
                ),
            }
        }
    }

    fn is_exported(attributes: &[syn::Attribute]) -> bool {
        attributes.iter().any(|attribute| {
            let tokens = format_tokens(&attribute.meta).replace(' ', "");
            tokens == "no_mangle"
                || tokens == "unsafe(no_mangle)"
                || tokens.starts_with("export_name=")
                || tokens.starts_with("unsafe(export_name=")
        })
    }

    fn is_host(item: &Item, host_items: &[String]) -> bool {
        match item {
            Item::Fn(function) => {
                host_items.contains(&function.sig.ident.to_string())
                    || function.sig.abi.is_some()
                    || Self::is_exported(&function.attrs)
            }
            Item::Static(item) => {
                host_items.contains(&item.ident.to_string()) || Self::is_exported(&item.attrs)
            }
            Item::Struct(item) => host_items.contains(&item.ident.to_string()),
            Item::Enum(item) => host_items.contains(&item.ident.to_string()),
            Item::Impl(item) => match item.self_ty.as_ref() {
                syn::Type::Path(path) => path
                    .path
                    .segments
                    .last()
                    .is_some_and(|segment| host_items.contains(&segment.ident.to_string())),
                _ => false,
            },
            _ => false,
        }
    }

    fn is_proc_macro(item: &Item) -> bool {
        let Item::Fn(function) = item else {
            return false;
        };
        if !matches!(function.vis, Visibility::Public(_)) {
            return false;
        }
        function.attrs.iter().any(|attribute| {
            ["proc_macro", "proc_macro_derive", "proc_macro_attribute"]
                .iter()
                .any(|name| attribute.path().is_ident(name))
        })
    }

    fn stage(
        item: &Item,
        is_entry: bool,
        is_bin: bool,
        is_macro: bool,
        hosts: &[String],
    ) -> Option<u8> {
        let allows_import = is_entry || is_bin || is_macro;
        match item {
            Item::Use(item) => {
                let is_private = matches!(item.vis, Visibility::Inherited);
                (allows_import && is_private).then_some(0)
            }
            Item::Mod(module) if module.content.is_some() => {
                Some(match module.ident.to_string().as_str() {
                    "fixture" => 3,
                    "test" => 4,
                    _ => 1,
                })
            }
            Item::Fn(function) if is_bin && function.sig.ident == "main" => Some(2),
            Item::Macro(definition) => definition.ident.is_none().then_some(1),
            _ if is_entry => Self::is_host(item, hosts).then_some(2),
            _ if is_macro => Self::is_proc_macro(item).then_some(2),
            _ => None,
        }
    }

    fn check_fixture(
        file: &SourceFile,
        item: &Item,
        module: &ItemMod,
        findings: &mut Vec<RuleFinding>,
    ) {
        if module.ident != "fixture" {
            return;
        }
        let attributes: Vec<String> = module
            .attrs
            .iter()
            .map(|attribute| format_tokens(&attribute.meta).replace(' ', ""))
            .collect();
        let is_public = matches!(module.vis, Visibility::Public(_));
        let has_cfg = attributes.contains(&"cfg(any(test,feature=\"fake\"))".to_owned());
        let has_path = attributes.contains(&"path=\"../fixture\"".to_owned());
        if !is_public || !has_cfg || !has_path {
            Self::report(
                file,
                item,
                "fixture is pub with cfg(any(test, feature = \"fake\")) and path = \"../fixture\"",
                findings,
            );
        }
    }

    fn check_test_block(
        file: &SourceFile,
        item: &Item,
        module: &ItemMod,
        findings: &mut Vec<RuleFinding>,
    ) {
        if module.ident != "test" {
            return;
        }
        let attributes: Vec<String> = module
            .attrs
            .iter()
            .map(|attribute| format_tokens(&attribute.meta).replace(' ', ""))
            .collect();
        let has_cfg = attributes.contains(&"cfg(test)".to_owned());
        let has_path = attributes.contains(&"path=\"../test\"".to_owned());
        let is_private = matches!(module.vis, Visibility::Inherited);
        if !has_cfg || !has_path || !is_private {
            Self::report(
                file,
                item,
                "test is private, cfg(test) with path = \"../test\"",
                findings,
            );
        }
    }

    fn check_binary_root(file: &SourceFile, findings: &mut Vec<RuleFinding>) {
        let mut saw_main = false;
        for item in &file.syntax.items {
            match item {
                Item::Use(import) if matches!(import.vis, Visibility::Inherited) => {
                    if saw_main {
                        Self::report(file, item, "use lines precede fn main", findings);
                    }
                }
                Item::Fn(function) if function.sig.ident == "main" => saw_main = true,
                _ => Self::report(
                    file,
                    item,
                    "src/main.rs of a crate with src/lib.rs holds only use items and fn main; the module tree is declared in src/lib.rs",
                    findings,
                ),
            }
        }
    }

    fn is_binary_root(file: &SourceFile, member: Option<&Member>) -> bool {
        member.is_some_and(|member| {
            member
                .targets
                .iter()
                .any(|target| target.src_path == file.path && target.is_bin())
        })
    }

    fn check_root(file: &SourceFile, context: &RuleContext, findings: &mut Vec<RuleFinding>) {
        let member = context
            .workspace
            .members
            .iter()
            .find(|member| member.name == file.module.crate_name);
        let is_bin = Self::is_binary_root(file, member);
        if is_bin && member.is_some_and(|member| member.targets.iter().any(Target::is_lib_target)) {
            Self::check_binary_root(file, findings);
            return;
        }
        let config = context
            .project_config
            .crates
            .iter()
            .find(|config| config.name == file.module.crate_name);
        let is_entry = config.is_some_and(|config| config.role == Some(Role::EntryPoint));
        let is_macro = config.is_some_and(|config| config.role == Some(Role::ProcMacro));
        let hosts = config.map_or(&[][..], |config| config.host_items.as_slice());
        let mut previous = 0;
        for item in &file.syntax.items {
            match Self::stage(item, is_entry, is_bin, is_macro, hosts) {
                Some(stage) if stage >= previous => previous = stage,
                Some(_) => Self::report(
                    file,
                    item,
                    "root items follow use, module tree, host items, fixture, test order",
                    findings,
                ),
                None => Self::report(
                    file,
                    item,
                    "the root file holds only the module tree and the items the platform calls",
                    findings,
                ),
            }
            if let Item::Mod(module) = item {
                Self::check_block(file, module, findings);
                Self::check_fixture(file, item, module, findings);
                Self::check_test_block(file, item, module, findings);
            }
        }
    }

    fn check_file(file: &SourceFile, context: &RuleContext, findings: &mut Vec<RuleFinding>) {
        if file.module.path.is_empty() {
            Self::check_root(file, context, findings);
            return;
        }
        for item in &file.syntax.items {
            if matches!(item, Item::Mod(_)) {
                Self::report(file, item, "only the crate root declares modules", findings);
            }
        }
    }
}

impl Rule for RootFileShape {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for sources in &context.crates {
            for file in &sources.files {
                Self::check_file(file, context, &mut findings);
            }
            for path in &sources.undeclared_files {
                findings.push(RuleFinding {
                    finding: Finding {
                        rule_id: RuleId("RootFileShape".to_owned()),
                        location: Location {
                            path: path.clone(),
                            line: None,
                        },
                        subject: path.clone(),
                        message: "no module tree declares this Rust source file".to_owned(),
                    },
                    anchor: Anchor::File(path.clone()),
                });
            }
        }
        findings
    }
}
