use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::check::util::format_module_path;
use crate::source::model::SourceFile;
use crate::source::util::find_node_span;
use crate::source::util::parse_macro_arguments;
use syn::ext::IdentExt;
use syn::visit::Visit;

pub(crate) struct NameVisitor<'source> {
    file: &'source SourceFile,
    rule: &'static str,
    technology_words: &'source [String],
    module_path: Vec<String>,
    is_trait_impl: bool,
    is_closure_parameter: bool,
    findings: Vec<RuleFinding>,
}

impl<'source> NameVisitor<'source> {
    fn new(
        file: &'source SourceFile,
        rule: &'static str,
        technology_words: &'source [String],
    ) -> Self {
        Self {
            file,
            rule,
            technology_words,
            module_path: file.module.path.clone(),
            is_trait_impl: false,
            is_closure_parameter: false,
            findings: Vec::new(),
        }
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

    fn check_type(&mut self, ident: &syn::Ident, is_trait: bool) {
        let name = ident.unraw().to_string();
        let has_bad_suffix = name.ends_with("Impl") || name.ends_with("Interface");
        let letters: Vec<char> = name.chars().take(3).collect();
        let has_interface_shape = matches!(letters.as_slice(), ['I', upper, lower] if upper.is_uppercase() && lower.is_lowercase());
        let has_interface_prefix = is_trait && has_interface_shape;
        if self.rule == "ImplementationName" && (has_bad_suffix || has_interface_prefix) {
            self.report_name(ident, "Name a type by its contract or implementation variant, without Impl, Interface or an I-prefix (arch-package.md)");
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
        if is_roleless || repeats_model_kind {
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
            path.is_ident("test")
                || (path.segments.len() == 2
                    && path.segments[0].ident == "tokio"
                    && path.segments[1].ident == "test")
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
        let is_exempt = is_lookup || is_test || self.is_trait_impl;
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
                Self::new(file, rule, words).collect_findings()
            })
            .collect()
    }
}

impl<'ast> Visit<'ast> for NameVisitor<'_> {
    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        for argument in &parse_macro_arguments(item) {
            self.visit_expr(argument);
        }
    }

    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        self.module_path.push(item.ident.unraw().to_string());
        syn::visit::visit_item_mod(self, item);
        self.module_path.pop();
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let parent = self.is_trait_impl;
        self.is_trait_impl = item.trait_.is_some();
        syn::visit::visit_item_impl(self, item);
        self.is_trait_impl = parent;
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        self.check_function(&item.sig, &item.attrs);
        syn::visit::visit_item_fn(self, item);
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
