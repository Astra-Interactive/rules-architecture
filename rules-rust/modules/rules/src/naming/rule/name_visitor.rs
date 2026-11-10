use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::check::util::format_module_path;
use crate::naming::model::SelectPart;
use crate::naming::util::parse_select_arguments;
use crate::source::model::SourceFile;
use crate::source::util::find_node_span;
use crate::source::util::parse_macro_arguments;
use syn::ext::IdentExt;
use syn::parse::ParseStream;
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct NameVisitor<'source> {
    context: &'source RuleContext,
    file: &'source SourceFile,
    rule: &'static str,
    technology_words: &'source [String],
    module_path: Vec<String>,
    scope: Option<ModuleScope>,
    is_trait_impl: bool,
    is_closure_parameter: bool,
    findings: Vec<RuleFinding>,
}

impl<'source> NameVisitor<'source> {
    fn find_scope(context: &RuleContext, file: &SourceFile) -> Option<ModuleScope> {
        let (ordinal, index) = context.indexes.iter().enumerate().find(|(_, index)| {
            index.crate_name == file.module.crate_name && index.target == file.module.target
        })?;
        let module = index
            .modules
            .iter()
            .find(|module| module.module == file.module)?;
        Some(ModuleScope {
            ordinal,
            module: module.id,
        })
    }

    fn new(
        context: &'source RuleContext,
        file: &'source SourceFile,
        rule: &'static str,
        technology_words: &'source [String],
    ) -> Self {
        Self {
            context,
            file,
            rule,
            technology_words,
            module_path: file.module.path.clone(),
            scope: Self::find_scope(context, file),
            is_trait_impl: false,
            is_closure_parameter: false,
            findings: Vec::new(),
        }
    }

    fn find_child_scope(&self, parent: ModuleScope, item: &syn::ItemMod) -> ModuleScope {
        let span = find_node_span(item.span());
        self.context.indexes[parent.ordinal]
            .modules
            .iter()
            .find(|module| module.parent == Some(parent.module) && module.span == span)
            .map_or(parent, |module| ModuleScope {
                ordinal: parent.ordinal,
                module: module.id,
            })
    }

    fn is_tokio_item(&self, path: &syn::Path, name: &str) -> bool {
        let is_written_out = path.segments.len() == 2
            && path.segments[0].ident == "tokio"
            && path.segments[1].ident == name;
        is_written_out
            || self.scope.is_some_and(|scope| {
                self.context
                    .resolve_external_call_paths(scope.ordinal, scope.module, path)
                    .iter()
                    .any(|resolved| resolved.format_path() == format!("tokio::{name}"))
            })
    }

    fn report_name(&mut self, ident: &syn::Ident, message: &str) {
        let module = format_module_path(&self.file.module.crate_name, &self.module_path);
        self.findings.push(build_node_finding(
            self.rule,
            &self.file.path,
            find_node_span(ident.span()),
            format!("{module}::{}", ident.unraw()),
            message.to_owned(),
        ));
    }

    fn check_declared_name(&mut self, ident: &syn::Ident) {
        if self.rule == "ExplicitItName" && ident.unraw() == "it" {
            self.report_name(
                ident,
                "Use a descriptive declaration or import name, never it (closure-naming.rs.md)",
            );
        }
    }

    fn check_import(&mut self, tree: &syn::UseTree, parent: Option<&syn::Ident>) {
        match tree {
            syn::UseTree::Path(path) => self.check_import(&path.tree, Some(&path.ident)),
            syn::UseTree::Name(name) if name.ident == "self" => {
                if let Some(parent) = parent {
                    self.check_declared_name(parent);
                }
            }
            syn::UseTree::Name(name) => self.check_declared_name(&name.ident),
            syn::UseTree::Rename(rename) => self.check_declared_name(&rename.rename),
            syn::UseTree::Group(group) => {
                for tree in &group.items {
                    self.check_import(tree, parent);
                }
            }
            _ => {}
        }
    }

    pub(crate) fn has_interface_prefix(name: &str) -> bool {
        let letters: Vec<char> = name.chars().take(3).collect();
        matches!(letters.as_slice(), ['I', upper, lower] if upper.is_uppercase() && lower.is_lowercase())
    }

    fn has_service_word(name: &str) -> bool {
        name.match_indices("Service").any(|(position, word)| {
            name[position + word.len()..]
                .chars()
                .next()
                .is_none_or(|next| !next.is_lowercase())
        })
    }

    fn check_type(&mut self, ident: &syn::Ident, is_trait: bool) {
        let name = ident.unraw().to_string();
        let has_bad_suffix = name.ends_with("Impl") || name.ends_with("Interface");
        let has_interface_prefix = is_trait && Self::has_interface_prefix(&name);
        if self.rule == "ImplementationName" && (has_bad_suffix || has_interface_prefix) {
            self.report_name(ident, Self::IMPLEMENTATION_NAME_MESSAGE);
        }
        if self.rule != "KindNameInPackage" {
            return;
        }
        let segments: Vec<&str> = self
            .file
            .module
            .crate_name
            .split('-')
            .chain(self.module_path.iter().map(String::as_str))
            .collect();
        let is_domain_model = segments
            .iter()
            .position(|segment| *segment == "model")
            .is_some_and(|position| {
                !segments[..position]
                    .iter()
                    .any(|segment| self.technology_words.iter().any(|word| word == segment))
            });
        let is_roleless = ["Helper", "Util", "Utils", "Manager"]
            .iter()
            .any(|suffix| name.ends_with(suffix));
        let repeats_model_kind =
            is_domain_model && (name.ends_with("Model") || name.ends_with("Entity"));
        let misplaced_service = !self.module_path.iter().any(|segment| segment == "service")
            && Self::has_service_word(&name);
        let vague_contract = is_trait
            && segments.contains(&"api")
            && ["Api", "Dao", "Krate", "Storage"]
                .iter()
                .any(|suffix| name.ends_with(suffix));
        if is_roleless || repeats_model_kind || misplaced_service || vague_contract {
            self.report_name(ident, "Name a type by its role without a utility suffix or a repeated domain model kind (clean-code.rs.md; arch-package.md)");
        }
    }

    fn is_test_name(name: &str) -> bool {
        let is_snake_case = name.bytes().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == b'_'
        });
        if !is_snake_case {
            return false;
        }
        let Some(state) = name.strip_prefix("given_") else {
            return false;
        };
        state.match_indices("_when_").any(|(position, _)| {
            if position == 0 {
                return false;
            }
            let action_text = &state[position + 6..];
            action_text.match_indices("_then_").any(|(action, _)| {
                let has_action = action > 0;
                let has_outcome = action + 6 < action_text.len();
                has_action && has_outcome
            })
        })
    }

    fn check_function(&mut self, signature: &syn::Signature, attributes: &[syn::Attribute]) {
        let name = signature.ident.unraw().to_string();
        let is_test = attributes.iter().any(|attribute| {
            let path = attribute.path();
            path.is_ident("test") || self.is_tokio_item(path, "test")
        });
        if self.rule == "TestFunctionName" && is_test && !Self::is_test_name(&name) {
            self.report_name(
                &signature.ident,
                "Use given_<state>_when_<action>_then_<outcome> for a test (testing.rs.md)",
            );
        }
        let is_lookup =
            ["get", "get_mut", "get_unchecked", "get_unchecked_mut"].contains(&name.as_str());
        let forbidden = ["get", "map", "make", "do", "handle", "process"]
            .contains(&name.split('_').next().unwrap_or(""));
        let checks_function_verb = self.rule == "FunctionVerb";
        let is_keyed_lookup =
            is_lookup && signature.receiver().is_some() && signature.inputs.len() > 1;
        let is_exempt = is_keyed_lookup || is_test || self.is_trait_impl;
        if checks_function_verb && forbidden && !is_exempt {
            self.report_name(&signature.ident, "Use a specific verb instead of get, map, make, do, handle or process (clean-code.rs.md)");
        }
    }

    fn check_field_name(&mut self, field: &syn::Field) {
        let Some(ident) = &field.ident else {
            return;
        };
        if ident.unraw() == "it" {
            self.report_name(
                ident,
                "Name the field after what it holds, never it (closure-naming.rs.md)",
            );
        }
    }

    fn try_visit_select(&mut self, item: &syn::Macro) -> bool {
        if !self.is_tokio_item(&item.path, "select") {
            return false;
        }
        let Ok(parts) = parse_select_arguments.parse2(item.tokens.clone()) else {
            return false;
        };
        for part in &parts {
            match part {
                SelectPart::Pattern(pattern) => self.visit_pat(pattern),
                SelectPart::Expression(expression) => self.visit_expr(expression),
            }
        }
        true
    }

    fn collect_findings(mut self) -> Vec<RuleFinding> {
        self.visit_file(&self.file.syntax);
        self.findings
    }

    pub(crate) fn check_context(
        context: &'source RuleContext,
        rule: &'static str,
    ) -> Vec<RuleFinding> {
        context
            .crates
            .iter()
            .flat_map(|sources| &sources.files)
            .flat_map(|file| {
                let words = context
                    .project_config
                    .crates
                    .iter()
                    .find(|config| config.name == file.module.crate_name)
                    .map(|config| config.technology_words.as_slice())
                    .unwrap_or(&[]);
                Self::new(context, file, rule, words).collect_findings()
            })
            .collect()
    }

    pub(crate) const IMPLEMENTATION_NAME_MESSAGE: &'static str = "Name a type by its contract or implementation variant, without Impl, Interface or an I-prefix (arch-package.md)";
}

impl<'ast> Visit<'ast> for NameVisitor<'_> {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        let name = match item {
            syn::Item::Const(item) => Some(&item.ident),
            syn::Item::Static(item) => Some(&item.ident),
            syn::Item::Fn(item) => Some(&item.sig.ident),
            syn::Item::Struct(item) => Some(&item.ident),
            syn::Item::Enum(item) => Some(&item.ident),
            syn::Item::Union(item) => Some(&item.ident),
            syn::Item::Trait(item) => Some(&item.ident),
            syn::Item::TraitAlias(item) => Some(&item.ident),
            syn::Item::Type(item) => Some(&item.ident),
            syn::Item::Mod(item) => Some(&item.ident),
            syn::Item::ExternCrate(item) => Some(
                item.rename
                    .as_ref()
                    .map_or(&item.ident, |(_, renamed)| renamed),
            ),
            syn::Item::Macro(item) => item.ident.as_ref(),
            _ => None,
        };
        if let Some(name) = name {
            self.check_declared_name(name);
        }
        syn::visit::visit_item(self, item);
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        self.check_import(&item.tree, None);
    }

    fn visit_foreign_item(&mut self, item: &'ast syn::ForeignItem) {
        match item {
            syn::ForeignItem::Fn(function) => self.check_declared_name(&function.sig.ident),
            syn::ForeignItem::Static(value) => self.check_declared_name(&value.ident),
            syn::ForeignItem::Type(typed) => self.check_declared_name(&typed.ident),
            _ => {}
        }
        syn::visit::visit_foreign_item(self, item);
    }

    fn visit_variant(&mut self, item: &'ast syn::Variant) {
        self.check_declared_name(&item.ident);
        syn::visit::visit_variant(self, item);
    }

    fn visit_generic_param(&mut self, parameter: &'ast syn::GenericParam) {
        match parameter {
            syn::GenericParam::Type(typed) => self.check_declared_name(&typed.ident),
            syn::GenericParam::Const(value) => self.check_declared_name(&value.ident),
            syn::GenericParam::Lifetime(lifetime) => {
                self.check_declared_name(&lifetime.lifetime.ident)
            }
        }
        syn::visit::visit_generic_param(self, parameter);
    }

    fn visit_trait_item(&mut self, item: &'ast syn::TraitItem) {
        match item {
            syn::TraitItem::Fn(function) => self.check_declared_name(&function.sig.ident),
            syn::TraitItem::Const(value) => self.check_declared_name(&value.ident),
            syn::TraitItem::Type(typed) => self.check_declared_name(&typed.ident),
            _ => {}
        }
        syn::visit::visit_trait_item(self, item);
    }

    fn visit_impl_item(&mut self, item: &'ast syn::ImplItem) {
        if !self.is_trait_impl {
            match item {
                syn::ImplItem::Fn(function) => self.check_declared_name(&function.sig.ident),
                syn::ImplItem::Const(value) => self.check_declared_name(&value.ident),
                syn::ImplItem::Type(typed) => self.check_declared_name(&typed.ident),
                _ => {}
            }
        }
        syn::visit::visit_impl_item(self, item);
    }

    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        let is_matches = item
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "matches");
        if is_matches && let Ok(arguments) = parse_matches_arguments.parse2(item.tokens.clone()) {
            self.visit_expr(&arguments.scrutinee);
            self.visit_pat(&arguments.pattern);
            if let Some(guard) = &arguments.guard {
                self.visit_expr(guard);
            }
            return;
        }
        if self.try_visit_select(item) {
            return;
        }
        for argument in &parse_macro_arguments(item) {
            self.visit_expr(argument);
        }
    }

    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        let parent_scope = self.scope;
        self.scope = parent_scope.map(|scope| self.find_child_scope(scope, item));
        self.module_path.push(item.ident.unraw().to_string());
        syn::visit::visit_item_mod(self, item);
        self.module_path.pop();
        self.scope = parent_scope;
    }

    fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
        self.check_type(&item.ident, false);
        syn::visit::visit_item_type(self, item);
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let parent = self.is_trait_impl;
        self.is_trait_impl = item.trait_.is_some();
        syn::visit::visit_item_impl(self, item);
        self.is_trait_impl = parent;
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        let previous = self.is_trait_impl;
        self.is_trait_impl = false;
        self.check_function(&item.sig, &item.attrs);
        syn::visit::visit_item_fn(self, item);
        self.is_trait_impl = previous;
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        self.check_function(&item.sig, &item.attrs);
        syn::visit::visit_impl_item_fn(self, item);
    }

    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        self.check_function(&item.sig, &item.attrs);
        syn::visit::visit_trait_item_fn(self, item);
    }

    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        self.check_type(&item.ident, false);
        syn::visit::visit_item_struct(self, item);
    }

    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        self.check_type(&item.ident, false);
        syn::visit::visit_item_enum(self, item);
    }

    fn visit_item_union(&mut self, item: &'ast syn::ItemUnion) {
        self.check_type(&item.ident, false);
        syn::visit::visit_item_union(self, item);
    }

    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        self.check_type(&item.ident, true);
        syn::visit::visit_item_trait(self, item);
    }

    fn visit_expr_closure(&mut self, expression: &'ast syn::ExprClosure) {
        self.is_closure_parameter = true;
        for input in &expression.inputs {
            self.visit_pat(input);
        }
        self.is_closure_parameter = false;
        self.visit_expr(&expression.body);
    }

    fn visit_pat_ident(&mut self, pattern: &'ast syn::PatIdent) {
        let name = pattern.ident.unraw().to_string();
        let single_letter = self.rule == "ClosureParameterName"
            && self.is_closure_parameter
            && !name.starts_with('_')
            && name.chars().count() == 1;
        if single_letter || (self.rule == "ExplicitItName" && name == "it") {
            self.report_name(&pattern.ident, "Give a binding a descriptive name, never it or a one-letter closure parameter (closure-naming.rs.md)");
        }
        syn::visit::visit_pat_ident(self, pattern);
    }

    fn visit_named_arg(&mut self, argument: &'ast syn::NamedArg) {
        if let Some((name, _)) = &argument.name {
            self.check_declared_name(name);
        }
        syn::visit::visit_named_arg(self, argument);
    }

    fn visit_field(&mut self, field: &'ast syn::Field) {
        if self.rule == "ExplicitItName" {
            self.check_field_name(field);
        }
        syn::visit::visit_field(self, field);
    }

    fn visit_label(&mut self, label: &'ast syn::Label) {
        if self.rule == "ExplicitItName" && label.name.ident == "it" {
            self.report_name(
                &label.name.ident,
                "Use a descriptive loop label, never it (closure-naming.rs.md)",
            );
        }
    }
}

#[derive(Clone, Copy)]
struct ModuleScope {
    ordinal: usize,
    module: usize,
}

struct MatchesArguments {
    scrutinee: syn::Expr,
    pattern: syn::Pat,
    guard: Option<syn::Expr>,
}

fn parse_matches_arguments(input: ParseStream) -> syn::Result<MatchesArguments> {
    let scrutinee: syn::Expr = input.parse()?;
    input.parse::<syn::Token![,]>()?;
    let pattern = syn::Pat::parse_multi_with_leading_vert(input)?;
    let mut guard = None;
    if input.peek(syn::Token![if]) {
        input.parse::<syn::Token![if]>()?;
        let condition: syn::Expr = input.parse()?;
        guard = Some(condition);
    }
    input.parse::<Option<syn::Token![,]>>()?;
    Ok(MatchesArguments {
        scrutinee,
        pattern,
        guard,
    })
}
