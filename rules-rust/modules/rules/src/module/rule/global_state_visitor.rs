use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::config::model::Role;
use crate::source::model::Resolution;
use crate::source::util::find_node_span;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct GlobalStateVisitor<'context> {
    context: &'context RuleContext,
    ordinal: usize,
    module: usize,
    file: String,
    is_main: bool,
    findings: Vec<RuleFinding>,
    builder_locals: Vec<String>,
}

impl GlobalStateVisitor<'_> {
    fn report(&mut self, span: proc_macro2::Span, message: &str) {
        self.findings.push(build_node_finding(
            "GlobalState",
            &self.file,
            find_node_span(span),
            self.context.indexes[self.ordinal].modules[self.module]
                .module
                .path
                .join("::"),
            message.to_owned(),
        ));
    }

    fn resolved_package(&self, path: &syn::Path) -> Option<String> {
        let Resolution::External(path) =
            self.context.indexes[self.ordinal].resolve_path(self.module, path)
        else {
            return None;
        };
        Some(
            self.context
                .resolve_external_identity(self.ordinal, &path)
                .package,
        )
    }

    fn is_thread_local_macro(&self, path: &syn::Path) -> bool {
        if path
            .segments
            .last()
            .is_none_or(|segment| segment.ident != "thread_local")
        {
            return false;
        }
        match self.context.indexes[self.ordinal].resolve_path(self.module, path) {
            Resolution::External(path) => {
                self.context
                    .resolve_external_identity(self.ordinal, &path)
                    .package
                    == "std"
            }
            Resolution::Unresolved => true,
            _ => false,
        }
    }

    fn parse_thread_local_statics(item: &syn::Macro) -> Vec<syn::ItemStatic> {
        let mut tokens = item.tokens.clone();
        let ends_with_semicolon =
            tokens
                .clone()
                .into_iter()
                .last()
                .is_some_and(|token| match token {
                    proc_macro2::TokenTree::Punct(punct) => punct.as_char() == ';',
                    _ => false,
                });
        if !ends_with_semicolon {
            tokens.extend([proc_macro2::TokenTree::Punct(proc_macro2::Punct::new(
                ';',
                proc_macro2::Spacing::Alone,
            ))]);
        }
        let parser = |stream: syn::parse::ParseStream| {
            let mut statics = Vec::new();
            while !stream.is_empty() {
                statics.push(stream.parse::<syn::ItemStatic>()?);
            }
            Ok(statics)
        };
        syn::parse::Parser::parse2(parser, tokens).unwrap_or_default()
    }

    fn report_thread_local_statics(&mut self, item: &syn::Macro) {
        let cfg = crate::source::model::CfgPredicate::All(Vec::new());
        for shared in Self::parse_thread_local_statics(item) {
            if Self::invalid_static(self.context, self.ordinal, self.module, &shared, &cfg) {
                self.report(
                    shared.span(),
                    "shared state is passed from DI, never stored in a global static",
                );
            }
        }
    }

    fn report_module_thread_locals(&mut self, items: &[syn::Item], module: usize) {
        self.module = module;
        for item in items {
            match item {
                syn::Item::Macro(call) if call.ident.is_none() => {
                    if self.is_thread_local_macro(&call.mac.path) {
                        self.report_thread_local_statics(&call.mac);
                    }
                }
                syn::Item::Mod(child) => {
                    let name = child.ident.to_string();
                    let child_module = self.context.indexes[self.ordinal]
                        .modules
                        .iter()
                        .find(|candidate| {
                            candidate.parent == Some(module)
                                && candidate.module.path.last() == Some(&name)
                        })
                        .map(|candidate| candidate.id);
                    if let (Some((_, nested)), Some(child_module)) = (&child.content, child_module)
                    {
                        self.report_module_thread_locals(nested, child_module);
                        self.module = module;
                    }
                }
                _ => {}
            }
        }
    }

    fn is_subscriber_builder(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::MethodCall(call) => self.is_subscriber_builder(&call.receiver),
            syn::Expr::Paren(paren) => self.is_subscriber_builder(&paren.expr),
            syn::Expr::Call(call) => {
                matches!(call.func.as_ref(), syn::Expr::Path(path) if self.resolved_package(&path.path).as_deref() == Some("tracing_subscriber"))
            }
            syn::Expr::Path(path) => path
                .path
                .get_ident()
                .is_some_and(|name| self.builder_locals.iter().any(|local| name == local)),
            _ => false,
        }
    }

    fn find_service_type<'typed>(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        typed: &'typed syn::Type,
    ) -> &'typed syn::Type {
        match typed {
            syn::Type::Reference(reference) => {
                Self::find_service_type(context, ordinal, module, &reference.elem)
            }
            syn::Type::Paren(paren) => {
                Self::find_service_type(context, ordinal, module, &paren.elem)
            }
            syn::Type::Path(path) => {
                let is_pointer = context
                    .canonical_external_type_path(ordinal, module, &path.path)
                    .is_some_and(|identity| {
                        ["std", "alloc"].contains(&identity.package.as_str())
                            && identity
                                .segments
                                .last()
                                .is_some_and(|name| ["Box", "Arc", "Rc"].contains(&name.as_str()))
                    });
                let inner =
                    path.path
                        .segments
                        .last()
                        .and_then(|segment| match &segment.arguments {
                            syn::PathArguments::AngleBracketed(arguments) => {
                                arguments.args.iter().find_map(|argument| match argument {
                                    syn::GenericArgument::Type(inner) => Some(inner),
                                    _ => None,
                                })
                            }
                            _ => None,
                        });
                match inner {
                    Some(inner) if is_pointer => {
                        Self::find_service_type(context, ordinal, module, inner)
                    }
                    _ => typed,
                }
            }
            other => other,
        }
    }

    fn is_workspace_trait_object(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        typed: &syn::Type,
    ) -> bool {
        let syn::Type::TraitObject(object) = typed else {
            return false;
        };
        object.bounds.iter().any(|bound| {
            matches!(bound, syn::TypeParamBound::Trait(bound)
                if !context.resolve_workspace_path(ordinal, module, &bound.path).is_empty())
        })
    }

    fn installs_logging(path: &crate::source::model::ExternalPath) -> bool {
        let name = path.segments.last().map(String::as_str).unwrap_or("");
        match path.package.as_str() {
            "tracing" | "tracing_core" => name == "set_global_default",
            "tracing_subscriber" => ["init", "try_init"].contains(&name),
            "log" => ["set_logger", "set_boxed_logger"].contains(&name),
            _ => false,
        }
    }

    fn is_shared_state_type(path: &crate::source::model::ExternalPath) -> bool {
        let name = path.segments.last().map(String::as_str).unwrap_or("");
        let standard = ["std", "core", "alloc"].contains(&path.package.as_str());
        let library = ["tokio", "parking_lot", "once_cell"].contains(&path.package.as_str());
        let state = [
            "OnceLock", "LazyLock", "Mutex", "RwLock", "Cell", "RefCell", "OnceCell", "Lazy",
        ]
        .contains(&name)
            || name.starts_with("Atomic");
        (standard || library) && state
    }

    pub(crate) fn invalid_static(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        shared: &syn::ItemStatic,
        cfg: &crate::source::model::CfgPredicate,
    ) -> bool {
        let state = context
            .resolve_external_type_paths_under(ordinal, module, &shared.ty, cfg)
            .iter()
            .any(Self::is_shared_state_type);
        let service_type = Self::find_service_type(context, ordinal, module, &shared.ty);
        let service = context
            .resolve_workspace_type(ordinal, module, service_type)
            .iter()
            .any(|target| {
                target.path.item.is_some_and(|identity| {
                    crate::module::util::workspace_implementation(context, target.index, identity)
                })
            });
        let trait_object = Self::is_workspace_trait_object(context, ordinal, module, service_type);
        state
            || service
            || trait_object
            || matches!(shared.mutability, syn::StaticMutability::Mut(_))
    }

    pub(crate) fn check(context: &RuleContext, ordinal: usize) -> Vec<RuleFinding> {
        let index = &context.indexes[ordinal];
        let mut visitor = GlobalStateVisitor {
            context,
            ordinal,
            module: 0,
            file: String::new(),
            is_main: false,
            findings: Vec::new(),
            builder_locals: Vec::new(),
        };
        let installs_logger = matches!(
            context
                .find_role_config(&index.crate_name)
                .and_then(|config| config.role),
            Some(Role::EntryPoint | Role::Tooling)
        );
        for item in &index.items {
            visitor.module = item.module;
            visitor.file = item.file.clone();
            visitor.is_main = item.name == "main" && item.module == 0 && installs_logger;
            visitor.visit_item(&item.syntax);
        }
        for implementation in &index.implementations {
            visitor.module = implementation.module;
            visitor.file = implementation.file.clone();
            visitor.is_main = false;
            visitor.visit_item_impl(&implementation.syntax);
        }
        visitor.is_main = false;
        for sources in context.crates.iter().filter(|sources| {
            sources.crate_name == index.crate_name && sources.target == index.target
        }) {
            for file in &sources.files {
                let root = index
                    .modules
                    .iter()
                    .position(|module| module.module.path == file.module.path);
                if let Some(root) = root {
                    visitor.file = file.path.clone();
                    visitor.report_module_thread_locals(&file.syntax.items, root);
                }
            }
        }
        visitor.findings
    }
}

impl<'ast> Visit<'ast> for GlobalStateVisitor<'_> {
    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        let main = self.is_main;
        self.is_main &= function.sig.ident == "main";
        let locals = std::mem::take(&mut self.builder_locals);
        syn::visit::visit_item_fn(self, function);
        self.builder_locals = locals;
        self.is_main = main;
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        syn::visit::visit_local(self, local);
        let syn::Pat::Ident(binding) = &local.pat else {
            return;
        };
        let is_builder = local
            .init
            .as_ref()
            .is_some_and(|init| self.is_subscriber_builder(&init.expr));
        if is_builder {
            self.builder_locals.push(binding.ident.to_string());
        }
    }
    fn visit_stmt(&mut self, statement: &'ast syn::Stmt) {
        if let syn::Stmt::Item(syn::Item::Static(local)) = statement {
            let cfg = crate::source::model::CfgPredicate::All(Vec::new());
            let is_invalid =
                Self::invalid_static(self.context, self.ordinal, self.module, local, &cfg);
            if is_invalid {
                self.report(
                    local.span(),
                    "shared state is passed from DI, never stored in a global static",
                );
            }
        }
        syn::visit::visit_stmt(self, statement);
    }

    fn visit_macro(&mut self, expression: &'ast syn::Macro) {
        if self.resolved_package(&expression.path).as_deref() == Some("lazy_static") {
            self.report(
                expression.span(),
                "lazy_static state is owned by DI instead of a singleton",
            );
        }
        if self.is_thread_local_macro(&expression.path) {
            self.report_thread_local_statics(expression);
            return;
        }
        for argument in &crate::source::util::parse_macro_arguments(expression) {
            self.visit_expr(argument);
        }
    }

    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if self.is_main {
            syn::visit::visit_expr_call(self, call);
            return;
        }
        if let syn::Expr::Path(path) = call.func.as_ref() {
            let Resolution::External(path) =
                self.context.indexes[self.ordinal].resolve_path(self.module, &path.path)
            else {
                syn::visit::visit_expr_call(self, call);
                return;
            };
            let path = self.context.resolve_external_identity(self.ordinal, &path);
            if Self::installs_logging(&path) {
                self.report(
                    call.span(),
                    "install the logging subscriber in entry-point main",
                );
            }
        }
        syn::visit::visit_expr_call(self, call);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        let installs = call.method == "init" || call.method == "try_init";
        if !self.is_main && installs && self.is_subscriber_builder(&call.receiver) {
            self.report(
                call.span(),
                "install the logging subscriber in entry-point main",
            );
        }
        syn::visit::visit_expr_method_call(self, call);
    }
}
