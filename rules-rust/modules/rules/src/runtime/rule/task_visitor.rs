use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::{build_node_finding, format_module_path};
use crate::declaration::util::is_technology;
use crate::runtime::rule::ClosureEffects;
use crate::runtime::util::expects_lint;
use crate::runtime::util::node_predicate;
use crate::source::model::{Resolution, SourceFile};
use crate::source::util::{find_node_span, parse_macro_arguments, parse_macro_expressions};
use crate::workspace::model::TomlValue;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct TaskVisitor<'source> {
    context: &'source RuleContext,
    ordinal: usize,
    module: usize,
    file: &'source SourceFile,
    rule: &'static str,
    findings: Vec<RuleFinding>,
    async_context: bool,
    drop_context: bool,
    callback_depth: usize,
    discard: bool,
    thread_expect: bool,
    handles: Vec<(String, proc_macro2::Span, bool)>,
    local_self: bool,
    closures: Vec<(String, Option<syn::ExprClosure>, usize)>,
    bindings: Vec<LocalBinding>,
    return_start: usize,
    self_type: Option<syn::Type>,
    loop_prefix: Option<LoopPrefix>,
    inherit_async: bool,
    test_function: bool,
}

impl TaskVisitor<'_> {
    fn build_callback_source_context(context: &RuleContext) -> RuleContext {
        let mut indexes = context.indexes.clone();
        let mut seen = std::collections::BTreeSet::new();
        for dependency in context
            .workspace
            .members
            .iter()
            .flat_map(|member| &member.dependencies)
        {
            let Some(path) = &dependency.path else {
                continue;
            };
            let root = context.workspace.root.join(path);
            if !seen.insert(root.clone()) {
                continue;
            }
            let workspace_member = context
                .workspace
                .members
                .iter()
                .any(|member| context.workspace.root.join(&member.directory) == root);
            if workspace_member {
                continue;
            }
            let manifest_path = root.join("Cargo.toml");
            let Some(manifest) = std::fs::read_to_string(&manifest_path)
                .ok()
                .and_then(|text| {
                    crate::workspace::util::parse_toml_file(&manifest_path.to_string_lossy(), &text)
                        .ok()
                })
            else {
                continue;
            };
            let source = manifest
                .root
                .find(&["lib", "path"])
                .and_then(TomlValue::as_str)
                .unwrap_or("src/lib.rs");
            let sources = crate::source::syn::ModuleTreeWalker::new(
                root,
                dependency.name.clone(),
                dependency.name.replace('-', "_"),
            )
            .walk_crate(source, crate::source::model::SourceTree::Src);
            indexes.push(crate::source::syn::CrateIndexBuilder::new(&sources, Vec::new()).build());
        }
        RuleContext {
            workspace: context.workspace.clone(),
            project_config: context.project_config.clone(),
            crates: context.crates.clone(),
            indexes,
            names: std::sync::OnceLock::new(),
            rustfmt: context.rustfmt.clone(),
        }
    }

    pub(crate) fn check_context(context: &RuleContext, rule: &'static str) -> Vec<RuleFinding> {
        let callback_context =
            (rule == "CallbackBridgeShape").then(|| Self::build_callback_source_context(context));
        let context = callback_context.as_ref().unwrap_or(context);
        let mut findings = Vec::new();
        for sources in &context.crates {
            for file in &sources.files {
                if file.is_generated {
                    continue;
                }
                let Some((ordinal, index)) =
                    context.indexes.iter().enumerate().find(|(_, index)| {
                        index.crate_name == file.module.crate_name
                            && index.target == file.module.target
                    })
                else {
                    continue;
                };
                let Some(module) = index
                    .modules
                    .iter()
                    .find(|module| module.module == file.module)
                else {
                    continue;
                };
                let mut visitor = TaskVisitor {
                    context,
                    ordinal,
                    module: module.id,
                    file,
                    rule,
                    findings: Vec::new(),
                    async_context: false,
                    drop_context: false,
                    callback_depth: 0,
                    discard: false,
                    thread_expect: expects_lint(&file.syntax.attrs, "disallowed_methods"),
                    handles: Vec::new(),
                    local_self: false,
                    closures: Vec::new(),
                    bindings: Vec::new(),
                    return_start: 0,
                    self_type: None,
                    loop_prefix: None,
                    inherit_async: false,
                    test_function: false,
                };
                visitor.visit_file(&file.syntax);
                findings.extend(visitor.findings);
            }
        }
        findings
    }

    fn report(&mut self, span: proc_macro2::Span) {
        let index = &self.context.indexes[self.ordinal];
        if !node_predicate(index, self.module, &self.file.path, find_node_span(span))
            .is_satisfiable()
        {
            return;
        }
        if self.findings.iter().any(|finding| matches!(&finding.anchor, crate::check::model::Anchor::Node { span: existing, .. } if *existing == find_node_span(span))) { return; }
        self.findings.push(build_node_finding(self.rule, &self.file.path, find_node_span(span),
            format_module_path(&index.crate_name, &index.modules[self.module].module.path),
            format!("Follow the {} async ownership/context requirement (async.rs.md; no-callbacks.rs.md)", self.rule)));
    }

    fn external(&self, path: &syn::Path) -> Option<String> {
        let Resolution::External(path) =
            self.context.indexes[self.ordinal].resolve_path(self.module, path)
        else {
            return None;
        };
        Some(
            self.context
                .resolve_external_identity(self.ordinal, &path)
                .format_path(),
        )
    }

    fn is_spawn_path(path: &str) -> bool {
        matches!(path, "tokio::spawn" | "tokio::task::spawn")
    }

    fn is_blocking_spawn_path(path: &str) -> bool {
        path == "tokio::task::spawn_blocking"
    }

    fn is_local_spawn_path(path: &str) -> bool {
        path == "tokio::task::spawn_local"
    }

    fn is_test_attribute(attribute: &syn::Attribute) -> bool {
        attribute
            .path()
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "test")
    }

    pub(crate) fn runs_off_async(path: &str) -> bool {
        matches!(
            path,
            "tokio::task::spawn_blocking" | "tokio::task::block_in_place" | "std::thread::spawn"
        )
    }

    fn is_foreign_crate(path: &crate::source::model::ExternalPath) -> bool {
        !matches!(path.package.as_str(), "std" | "core" | "alloc")
    }

    fn is_boxed_closure(&self, argument: &syn::Expr) -> bool {
        let syn::Expr::Call(call) = argument else {
            return false;
        };
        let syn::Expr::Path(path) = &*call.func else {
            return false;
        };
        matches!(call.args.first(), Some(syn::Expr::Closure(_)))
            && self
                .context
                .resolve_external_call_paths(self.ordinal, self.module, &path.path)
                .iter()
                .any(|path| {
                    matches!(
                        path.format_path().as_str(),
                        "std::boxed::Box::new" | "alloc::boxed::Box::new"
                    )
                })
    }

    fn boxed_to_foreign_function(&self, call: &syn::ExprCall, argument: &syn::Expr) -> bool {
        let syn::Expr::Path(path) = &*call.func else {
            return false;
        };
        self.is_boxed_closure(argument)
            && self
                .context
                .resolve_external_call_paths(self.ordinal, self.module, &path.path)
                .iter()
                .any(Self::is_foreign_crate)
    }

    fn receiver_type(&self, expression: &syn::Expr) -> Option<syn::Type> {
        match expression {
            syn::Expr::Path(path) => self
                .bindings
                .iter()
                .rev()
                .find(|binding| path.path.is_ident(&binding.name))
                .and_then(|binding| binding.typed.clone()),
            syn::Expr::Reference(reference) => self.receiver_type(&reference.expr),
            syn::Expr::Paren(paren) => self.receiver_type(&paren.expr),
            syn::Expr::MethodCall(call) if call.method == "clone" => {
                self.receiver_type(&call.receiver)
            }
            syn::Expr::Field(field) if matches!(&*field.base, syn::Expr::Path(path) if path.path.is_ident("self")) =>
            {
                let typed = self.self_type.as_ref()?;
                self.context
                    .resolve_workspace_type(self.ordinal, self.module, typed)
                    .iter()
                    .find_map(|target| {
                        let item = &self.context.indexes[target.index].items[target.path.item?];
                        let syn::Item::Struct(item) = &item.syntax else {
                            return None;
                        };
                        item.fields
                            .iter()
                            .enumerate()
                            .find_map(|(position, entry)| {
                                let matching = match &field.member {
                                    syn::Member::Named(name) => entry.ident.as_ref() == Some(name),
                                    syn::Member::Unnamed(index) => position == index.index as usize,
                                };
                                matching.then(|| entry.ty.clone())
                            })
                    })
            }
            _ => None,
        }
    }

    fn boxed_to_foreign_method(&self, call: &syn::ExprMethodCall, argument: &syn::Expr) -> bool {
        self.is_boxed_closure(argument)
            && self.receiver_type(&call.receiver).is_some_and(|typed| {
                self.context
                    .resolve_workspace_type(self.ordinal, self.module, &typed)
                    .is_empty()
                    && self
                        .context
                        .resolve_external_type_paths(self.ordinal, self.module, &typed)
                        .iter()
                        .any(Self::is_foreign_crate)
            })
    }

    fn has_external_type(&self, typed: &syn::Type, names: &[&str]) -> bool {
        self.context
            .resolve_outer_external_type_paths(self.ordinal, self.module, typed)
            .iter()
            .any(|path| names.contains(&path.format_path().as_str()))
    }

    fn is_runtime_handle_call(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Paren(paren) => self.is_runtime_handle_call(&paren.expr),
            syn::Expr::Group(group) => self.is_runtime_handle_call(&group.expr),
            syn::Expr::Call(call) => {
                let syn::Expr::Path(path) = &*call.func else {
                    return false;
                };
                self.context
                    .resolve_external_call_paths(self.ordinal, self.module, &path.path)
                    .iter()
                    .any(|path| path.format_path() == "tokio::runtime::Handle::current")
            }
            syn::Expr::MethodCall(call) => {
                call.method == "handle"
                    && self.receiver_type(&call.receiver).is_some_and(|typed| {
                        self.has_external_type(&typed, &["tokio::runtime::Runtime"])
                    })
            }
            _ => false,
        }
    }

    fn is_runtime_spawn(&self, call: &syn::ExprMethodCall) -> bool {
        let receiver_has = |names: &[&str]| {
            self.receiver_type(&call.receiver)
                .is_some_and(|typed| self.has_external_type(&typed, names))
        };
        match call.method.to_string().as_str() {
            "spawn" | "spawn_blocking" => {
                receiver_has(&["tokio::runtime::Handle", "tokio::runtime::Runtime"])
                    || self.is_runtime_handle_call(&call.receiver)
            }
            "spawn_local" => receiver_has(&["tokio::task::LocalSet"]),
            _ => false,
        }
    }

    fn is_block_on_in_drop(&self, call: &syn::ExprMethodCall) -> bool {
        if self.rule != "BridgeCleanup" || !self.drop_context || call.method != "block_on" {
            return false;
        }
        self.is_runtime_handle_call(&call.receiver)
            || self.receiver_type(&call.receiver).is_some_and(|typed| {
                self.has_external_type(
                    &typed,
                    &["tokio::runtime::Handle", "tokio::runtime::Runtime"],
                )
            })
    }

    fn constructed_type(expression: &syn::Expr) -> Option<syn::Type> {
        let syn::Expr::Call(call) = expression else {
            return None;
        };
        let syn::Expr::Path(path) = &*call.func else {
            return None;
        };
        if path.path.segments.len() < 2 {
            return None;
        }
        let mut owner = path.path.clone();
        owner.segments.pop();
        owner.segments.pop_punct();
        Some(syn::Type::Path(syn::TypePath {
            attrs: Vec::new(),
            qself: path.qself.clone(),
            path: owner,
        }))
    }

    fn is_thread_builder_spawn(&self, call: &syn::ExprMethodCall) -> bool {
        if call.method != "spawn" {
            return false;
        }
        let mut receiver = &*call.receiver;
        while let syn::Expr::MethodCall(inner) = receiver
            && matches!(inner.method.to_string().as_str(), "name" | "stack_size")
        {
            receiver = &inner.receiver;
        }
        Self::constructed_type(receiver)
            .or_else(|| self.receiver_type(receiver))
            .is_some_and(|typed| self.has_external_type(&typed, &["std::thread::Builder"]))
    }

    fn is_thread_spawn(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Paren(paren) => self.is_thread_spawn(&paren.expr),
            syn::Expr::Group(group) => self.is_thread_spawn(&group.expr),
            syn::Expr::Try(attempt) => self.is_thread_spawn(&attempt.expr),
            syn::Expr::MethodCall(call)
                if matches!(call.method.to_string().as_str(), "unwrap" | "expect") =>
            {
                self.is_thread_spawn(&call.receiver)
            }
            syn::Expr::MethodCall(call) => self.is_thread_builder_spawn(call),
            syn::Expr::Call(call) => matches!(&*call.func, syn::Expr::Path(path)
                if self.external(&path.path).as_deref() == Some("std::thread::spawn")),
            _ => false,
        }
    }

    fn is_technology_module(&self) -> bool {
        is_technology(
            self.context,
            &self.context.indexes[self.ordinal],
            self.module,
        )
    }

    fn is_underscore_assignment(expression: &syn::ExprAssign) -> bool {
        matches!(&*expression.left, syn::Expr::Infer(_))
    }

    fn is_discard_transparent(expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Call(_)
            | syn::Expr::MethodCall(_)
            | syn::Expr::Try(_)
            | syn::Expr::Paren(_)
            | syn::Expr::Group(_) => true,
            syn::Expr::Assign(assignment) => Self::is_underscore_assignment(assignment),
            _ => false,
        }
    }

    fn is_dropped_thread_spawn(&self, is_drop: bool, argument: &syn::Expr) -> bool {
        if self.rule != "ThreadPrimitive" || !is_drop {
            return false;
        }
        self.is_technology_module() && self.is_thread_spawn(argument)
    }

    fn is_discarded_builder_spawn(&self, call: &syn::ExprMethodCall) -> bool {
        if self.rule != "ThreadPrimitive" || !self.discard {
            return false;
        }
        self.is_thread_builder_spawn(call) && self.is_technology_module()
    }

    fn is_spawn(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Paren(paren) => self.is_spawn(&paren.expr),
            syn::Expr::Group(group) => self.is_spawn(&group.expr),
            syn::Expr::Call(call) => matches!(&*call.func, syn::Expr::Path(path)
                if self.external(&path.path).is_some_and(|path| Self::is_spawn_path(&path) || Self::is_blocking_spawn_path(&path) || Self::is_local_spawn_path(&path))),
            syn::Expr::MethodCall(call) => self.is_runtime_spawn(call),
            _ => false,
        }
    }

    fn is_workspace_receiver(&self, receiver: &syn::Expr) -> bool {
        self.receiver_type(receiver).is_some_and(|typed| {
            !self
                .context
                .resolve_workspace_type(self.ordinal, self.module, &typed)
                .is_empty()
        })
    }

    fn is_local_name(&self, path: &syn::Path) -> bool {
        self.closures.iter().any(|(name, _, _)| path.is_ident(name))
    }

    fn find_local_closure(&self, expression: &syn::Expr) -> Option<&syn::ExprClosure> {
        match expression {
            syn::Expr::Paren(paren) => self.find_local_closure(&paren.expr),
            syn::Expr::Group(group) => self.find_local_closure(&group.expr),
            syn::Expr::Path(path) => self
                .closures
                .iter()
                .rev()
                .find(|(name, _, _)| path.path.is_ident(name))
                .and_then(|(_, closure, _)| closure.as_ref()),
            syn::Expr::Call(call) => call
                .args
                .iter()
                .find_map(|argument| self.find_local_closure(argument)),
            _ => None,
        }
    }

    fn runs_local_logic(&self, effects: &ClosureEffects) -> bool {
        effects.inline_logic
            || self.local_self && effects.methods.iter().any(|call| matches!(&*call.receiver, syn::Expr::Path(path) if path.path.is_ident("self")))
            || effects
                .methods
                .iter()
                .any(|call| self.is_workspace_receiver(&call.receiver))
            || effects.calls.iter().any(|call| matches!(&*call.func, syn::Expr::Path(path)
                if !self.is_local_name(&path.path)
                    && matches!(self.context.indexes[self.ordinal].resolve_path(self.module, &path.path), Resolution::Internal(_))))
    }

    fn runs_local_closure_logic(&self, argument: &syn::Expr) -> bool {
        self.find_local_closure(argument).is_some_and(|closure| {
            self.runs_local_logic(&ClosureEffects::collect(&syn::Expr::Closure(
                closure.clone(),
            )))
        })
    }

    fn is_workspace_function_reference(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Paren(paren) => self.is_workspace_function_reference(&paren.expr),
            syn::Expr::Group(group) => self.is_workspace_function_reference(&group.expr),
            syn::Expr::Path(path) if self.is_local_name(&path.path) => false,
            syn::Expr::Path(path) => {
                let index = &self.context.indexes[self.ordinal];
                let Resolution::Internal(targets) = index.resolve_path(self.module, &path.path)
                else {
                    return false;
                };
                targets.iter().any(|target| {
                    target
                        .item
                        .and_then(|identity| index.items.get(identity))
                        .is_some_and(|item| matches!(item.syntax, syn::Item::Fn(_)))
                })
            }
            syn::Expr::Call(call) => {
                let syn::Expr::Path(path) = &*call.func else {
                    return false;
                };
                self.external(&path.path).is_some_and(|path| {
                    matches!(
                        path.as_str(),
                        "std::panic::AssertUnwindSafe" | "core::panic::AssertUnwindSafe"
                    )
                }) && call
                    .args
                    .iter()
                    .any(|argument| self.is_workspace_function_reference(argument))
            }
            _ => false,
        }
    }

    fn check_unwind(&mut self, expression: &syn::ExprCall, canonical: Option<&str>) {
        if self.rule != "CatchUnwind"
            || !canonical.is_some_and(|path| {
                matches!(
                    path,
                    "std::panic::catch_unwind" | "core::panic::catch_unwind"
                )
            })
        {
            return;
        }
        let calls_local = expression.args.iter().any(|argument| {
            self.is_workspace_function_reference(argument)
                || self.runs_local_logic(&ClosureEffects::collect(argument))
                || self.runs_local_closure_logic(argument)
        });
        if calls_local {
            self.report(expression.span());
        }
    }

    fn discarded_guard(&self, expression: &syn::Expr) -> bool {
        let Some(typed) = Self::constructed_type(expression) else {
            return false;
        };
        self.context
            .resolve_workspace_type(self.ordinal, self.module, &typed)
            .iter()
            .any(|target| {
                let Some(identity) = target.path.item else {
                    return false;
                };
                let index = &self.context.indexes[target.index];
                let syn::Expr::Call(call) = expression else {
                    return false;
                };
                let syn::Expr::Path(path) = &*call.func else {
                    return false;
                };
                let Some(method) = path.path.segments.last() else {
                    return false;
                };
                let constructors =
                    self.guard_constructors(target.index, identity, &method.ident.to_string());
                if constructors.is_empty() {
                    return false;
                }
                index.implementations.iter().any(|implementation| {
                    let Some((path, _)) = &implementation.syntax.trait_ else {
                        return false;
                    };
                    let is_drop = self
                        .context
                        .resolve_external_call_paths(target.index, implementation.module, path)
                        .iter()
                        .any(|identity| {
                            matches!(
                                identity.format_path().as_str(),
                                "core::ops::Drop" | "std::ops::Drop"
                            )
                        });
                    is_drop
                        && constructors.iter().any(|constructor| {
                            target
                                .path
                                .cfg
                                .and(constructor)
                                .and(&implementation.cfg)
                                .is_satisfiable()
                        })
                        && index
                            .resolve_type_items(
                                implementation.module,
                                &implementation.syntax.self_ty,
                            )
                            .contains(&identity)
                })
            })
    }

    fn guard_constructors(
        &self,
        ordinal: usize,
        identity: usize,
        name: &str,
    ) -> Vec<crate::source::model::CfgPredicate> {
        let index = &self.context.indexes[ordinal];
        let mut constructors = Vec::new();
        for implementation in &index.implementations {
            if !index
                .resolve_type_items(implementation.module, &implementation.syntax.self_ty)
                .contains(&identity)
            {
                continue;
            }
            for item in &implementation.syntax.items {
                let syn::ImplItem::Fn(function) = item else {
                    continue;
                };
                if function.sig.receiver().is_some() || function.sig.ident != name {
                    continue;
                }
                let syn::ReturnType::Type(_, returned) = &function.sig.output else {
                    continue;
                };
                let cfg =
                    implementation
                        .cfg
                        .and(&crate::source::model::CfgPredicate::from_attributes(
                            &function.attrs,
                        ));
                constructors.extend(self.owned_return_configs(
                    ordinal,
                    implementation.module,
                    returned,
                    identity,
                    &cfg,
                ));
            }
        }
        constructors
    }

    fn owned_return_configs(
        &self,
        ordinal: usize,
        module: usize,
        returned: &syn::Type,
        identity: usize,
        cfg: &crate::source::model::CfgPredicate,
    ) -> Vec<crate::source::model::CfgPredicate> {
        self.context
            .resolve_type_syntax(ordinal, module, returned)
            .iter()
            .filter_map(|resolved| {
                let syn::Type::Path(path) = &resolved.typed else {
                    return None;
                };
                let own = path.path.is_ident("Self")
                    || self
                        .context
                        .resolve_outer_workspace_type(
                            resolved.index,
                            resolved.module,
                            &resolved.typed,
                        )
                        .iter()
                        .any(|target| {
                            target.index == ordinal && target.path.item == Some(identity)
                        });
                own.then(|| cfg.and(&resolved.cfg))
            })
            .collect()
    }

    fn is_owning_call(&self, call: &syn::ExprCall) -> bool {
        let syn::Expr::Path(path) = call.func.as_ref() else {
            return false;
        };
        path.path.is_ident("Some")
            && matches!(
                self.context.indexes[self.ordinal].resolve_path(self.module, &path.path),
                Resolution::Local
            )
            || self
                .context
                .resolve_external_call_paths(self.ordinal, self.module, &path.path)
                .iter()
                .any(|path| {
                    matches!(
                        path.format_path().as_str(),
                        "std::option::Option::Some"
                            | "core::option::Option::Some"
                            | "std::boxed::Box::new"
                            | "alloc::boxed::Box::new"
                    )
                })
    }

    fn is_vec_macro(&self, mac: &syn::Macro) -> bool {
        self.context
            .resolve_external_call_paths(self.ordinal, self.module, &mac.path)
            .iter()
            .any(|path| matches!(path.format_path().as_str(), "std::vec" | "alloc::vec"))
            || mac.path.is_ident("vec")
                && matches!(
                    self.context.indexes[self.ordinal].resolve_path(self.module, &mac.path),
                    Resolution::Local
                )
    }

    fn resolve_handles(&self, expression: &syn::Expr) -> Vec<usize> {
        match expression {
            syn::Expr::Path(path) => self
                .bindings
                .iter()
                .rev()
                .find(|binding| path.path.is_ident(&binding.name))
                .map(|binding| binding.handles.clone())
                .unwrap_or_default(),
            syn::Expr::Call(call) if self.is_owning_call(call) => call
                .args
                .iter()
                .flat_map(|argument| self.resolve_handles(argument))
                .collect(),
            syn::Expr::Array(array) => array
                .elems
                .iter()
                .flat_map(|element| self.resolve_handles(element))
                .collect(),
            syn::Expr::Macro(mac) if self.is_vec_macro(&mac.mac) => parse_macro_arguments(&mac.mac)
                .iter()
                .flat_map(|argument| self.resolve_handles(argument))
                .collect(),
            syn::Expr::Paren(paren) => self.resolve_handles(&paren.expr),
            syn::Expr::Group(group) => self.resolve_handles(&group.expr),
            _ => Vec::new(),
        }
    }

    fn collect_owned_handles(
        &mut self,
        expression: &syn::Expr,
        span: proc_macro2::Span,
    ) -> Vec<usize> {
        if self.is_spawn(expression) {
            let identity = self.handles.len();
            self.handles.push((String::new(), span, false));
            return vec![identity];
        }
        match expression {
            syn::Expr::Call(call) if self.is_owning_call(call) => call
                .args
                .iter()
                .flat_map(|argument| self.collect_owned_handles(argument, argument.span()))
                .collect(),
            syn::Expr::Array(array) => array
                .elems
                .iter()
                .flat_map(|element| self.collect_owned_handles(element, element.span()))
                .collect(),
            syn::Expr::Macro(mac) if self.is_vec_macro(&mac.mac) => parse_macro_arguments(&mac.mac)
                .iter()
                .flat_map(|argument| self.collect_owned_handles(argument, argument.span()))
                .collect(),
            syn::Expr::Paren(paren) => self.collect_owned_handles(&paren.expr, span),
            syn::Expr::Group(group) => self.collect_owned_handles(&group.expr, span),
            _ => self.resolve_handles(expression),
        }
    }

    fn resolve_ordered_handles(&self, expression: &syn::Expr) -> Option<Vec<usize>> {
        match expression {
            syn::Expr::Array(array) => array
                .elems
                .iter()
                .map(|element| {
                    let identities = self.resolve_handles(element);
                    (identities.len() == 1).then(|| identities[0])
                })
                .collect(),
            syn::Expr::Path(path) => self
                .bindings
                .iter()
                .rev()
                .find(|binding| path.path.is_ident(&binding.name))
                .and_then(|binding| binding.is_ordered.then(|| binding.handles.clone())),
            syn::Expr::Paren(paren) => self.resolve_ordered_handles(&paren.expr),
            syn::Expr::Group(group) => self.resolve_ordered_handles(&group.expr),
            _ => None,
        }
    }

    fn has_ordered_handles(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Array(array) => array
                .elems
                .iter()
                .all(|element| self.is_spawn(element) || self.resolve_handles(element).len() == 1),
            syn::Expr::Paren(paren) => self.has_ordered_handles(&paren.expr),
            syn::Expr::Group(group) => self.has_ordered_handles(&group.expr),
            _ => self.resolve_ordered_handles(expression).is_some(),
        }
    }

    fn is_straight_line_expression(expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Path(_) | syn::Expr::Lit(_) => true,
            syn::Expr::Await(awaited) => Self::is_straight_line_expression(&awaited.base),
            syn::Expr::Reference(reference) => Self::is_straight_line_expression(&reference.expr),
            syn::Expr::Paren(paren) => Self::is_straight_line_expression(&paren.expr),
            syn::Expr::Group(group) => Self::is_straight_line_expression(&group.expr),
            syn::Expr::Call(call) => {
                Self::is_straight_line_expression(&call.func)
                    && call.args.iter().all(Self::is_straight_line_expression)
            }
            syn::Expr::MethodCall(call) => {
                Self::is_straight_line_expression(&call.receiver)
                    && call.args.iter().all(Self::is_straight_line_expression)
            }
            syn::Expr::Block(block) => {
                block.label.is_none()
                    && block
                        .block
                        .stmts
                        .iter()
                        .all(Self::is_straight_line_statement)
            }
            _ => false,
        }
    }

    fn is_straight_line_statement(statement: &syn::Stmt) -> bool {
        match statement {
            syn::Stmt::Local(local) => local.init.as_ref().is_none_or(|init| {
                init.diverge.is_none() && Self::is_straight_line_expression(&init.expr)
            }),
            syn::Stmt::Expr(expression, _) => Self::is_straight_line_expression(expression),
            _ => false,
        }
    }

    fn shadow_pattern(&mut self, pattern: &syn::Pat) {
        match pattern {
            syn::Pat::Ident(name) => {
                self.bindings.push(LocalBinding {
                    name: name.ident.to_string(),
                    handles: Vec::new(),
                    typed: None,
                    is_ordered: false,
                });
                let environment = self.closures.len();
                self.closures
                    .push((name.ident.to_string(), None, environment));
                if let Some((_, pattern)) = &name.subpat {
                    self.shadow_pattern(pattern);
                }
            }
            syn::Pat::Type(pattern) => self.shadow_pattern(&pattern.pat),
            syn::Pat::Paren(pattern) => self.shadow_pattern(&pattern.pat),
            syn::Pat::Reference(pattern) => self.shadow_pattern(&pattern.pat),
            syn::Pat::Tuple(pattern) => {
                for pattern in &pattern.elems {
                    self.shadow_pattern(pattern);
                }
            }
            syn::Pat::TupleStruct(pattern) => {
                for pattern in &pattern.elems {
                    self.shadow_pattern(pattern);
                }
            }
            syn::Pat::Slice(pattern) => {
                for pattern in &pattern.elems {
                    self.shadow_pattern(pattern);
                }
            }
            syn::Pat::Struct(pattern) => {
                for field in &pattern.fields {
                    self.shadow_pattern(&field.pat);
                }
            }
            syn::Pat::Or(pattern) => {
                for pattern in &pattern.cases {
                    self.shadow_pattern(pattern);
                }
            }
            _ => syn::visit::visit_pat(self, pattern),
        }
    }

    fn pattern_name(pattern: &syn::Pat) -> Option<&syn::Ident> {
        match pattern {
            syn::Pat::Ident(name) => Some(&name.ident),
            syn::Pat::Type(pattern) => Self::pattern_name(&pattern.pat),
            syn::Pat::Paren(pattern) => Self::pattern_name(&pattern.pat),
            _ => None,
        }
    }

    fn record_loop_consumption(&mut self) {
        if let Some(loop_prefix) = &mut self.loop_prefix {
            loop_prefix.consumed = self
                .handles
                .iter()
                .map(|(_, _, consumed)| *consumed)
                .collect();
        }
    }

    fn visit_ordered_array(&mut self, expression: &syn::ExprForLoop, identities: Vec<usize>) {
        let prefix = expression
            .body
            .stmts
            .iter()
            .take_while(|statement| Self::is_straight_line_statement(statement))
            .count();
        let parent = self.loop_prefix.take();
        for identity in identities {
            let bindings = self.bindings.len();
            let closures = self.closures.len();
            let handles = self.handles.clone();
            self.shadow_pattern(&expression.pat);
            let binding = Self::pattern_name(&expression.pat).and_then(|name| {
                self.bindings
                    .iter_mut()
                    .rev()
                    .find(|binding| *name == binding.name)
            });
            if let Some(binding) = binding {
                binding.handles = vec![identity];
            }
            self.loop_prefix = Some(LoopPrefix {
                body: std::ptr::from_ref(&expression.body) as usize,
                prefix,
                consumed: handles.iter().map(|(_, _, consumed)| *consumed).collect(),
            });
            self.visit_block(&expression.body);
            let consumed = self
                .loop_prefix
                .take()
                .map(|loop_prefix| loop_prefix.consumed)
                .unwrap_or_default();
            self.bindings.truncate(bindings);
            self.closures.truncate(closures);
            self.handles = handles;
            for (handle, consumed) in self.handles.iter_mut().zip(consumed) {
                handle.2 |= consumed;
            }
            if prefix != expression.body.stmts.len() {
                break;
            }
        }
        self.loop_prefix = parent;
    }

    fn consume_handle(&mut self, expression: &syn::Expr) {
        for identity in self.resolve_handles(expression) {
            self.handles[identity].2 = true;
        }
    }

    fn bind_channel_sender(&mut self, local: &syn::Local) {
        let syn::Pat::Tuple(pattern) = &local.pat else {
            return;
        };
        let Some(name) = pattern.elems.first().and_then(Self::pattern_name) else {
            return;
        };
        let Some(init) = &local.init else {
            return;
        };
        let syn::Expr::Call(call) = &*init.expr else {
            return;
        };
        let syn::Expr::Path(path) = &*call.func else {
            return;
        };
        if !self
            .context
            .resolve_external_call_paths(self.ordinal, self.module, &path.path)
            .iter()
            .any(|path| path.format_path() == "tokio::sync::mpsc::channel")
        {
            return;
        }
        if let Some(binding) = self
            .bindings
            .iter_mut()
            .rev()
            .find(|binding| *name == binding.name)
        {
            binding.typed = Some(syn::parse_quote!(tokio::sync::mpsc::Sender<_>));
        }
    }

    fn bind_local(&mut self, local: &syn::Local) {
        let typed = match &local.pat {
            syn::Pat::Type(pattern) => Some((*pattern.ty).clone()),
            _ => local
                .init
                .as_ref()
                .and_then(|init| self.receiver_type(&init.expr)),
        };
        let handle = local
            .init
            .as_ref()
            .map(|init| {
                if self.rule == "DetachedTask" && Self::pattern_name(&local.pat).is_some() {
                    self.collect_owned_handles(&init.expr, local.span())
                } else {
                    self.resolve_handles(&init.expr)
                }
            })
            .unwrap_or_default();
        let ordered = local
            .init
            .as_ref()
            .is_some_and(|init| self.has_ordered_handles(&init.expr));
        self.shadow_pattern(&local.pat);
        self.bind_channel_sender(local);
        let Some(name) = Self::pattern_name(&local.pat) else {
            return;
        };
        if let Some(binding) = self
            .bindings
            .iter_mut()
            .rev()
            .find(|binding| *name == binding.name)
        {
            binding.handles = handle;
            binding.typed = typed;
            binding.is_ordered = ordered;
        }
        if matches!(self.rule, "CallbackBridgeShape" | "CatchUnwind") {
            let closure = local.init.as_ref().and_then(|init| match &*init.expr {
                syn::Expr::Closure(closure) => Some(closure.clone()),
                _ => None,
            });
            if let Some((_, bound, _)) = self
                .closures
                .iter_mut()
                .rev()
                .find(|(binding, _, _)| *name == binding)
            {
                *bound = closure;
            }
        }
    }

    fn callable_bound(&self, ordinal: usize, module: usize, bound: &syn::TypeParamBound) -> bool {
        let syn::TypeParamBound::Trait(bound) = bound else {
            return false;
        };
        (bound.path.segments.len() == 1
            && bound.path.segments.first().is_some_and(|segment| {
                ["Fn", "FnMut", "FnOnce"].contains(&segment.ident.to_string().as_str())
            }))
            && matches!(
                self.context.indexes[ordinal].resolve_path(module, &bound.path),
                Resolution::Local
            )
            || self
                .context
                .resolve_external_call_paths(ordinal, module, &bound.path)
                .iter()
                .any(|path| {
                    ["std", "core"].contains(&path.package.as_str())
                        && path
                            .segments
                            .last()
                            .is_some_and(|name| ["Fn", "FnMut", "FnOnce"].contains(&name.as_str()))
                })
    }

    fn carries_parameter(
        &self,
        ordinal: usize,
        module: usize,
        expression: &syn::Expr,
        name: &syn::Ident,
    ) -> bool {
        match expression {
            syn::Expr::Path(path) => path.path.is_ident(name),
            syn::Expr::Call(call) => {
                let syn::Expr::Path(path) = &*call.func else {
                    return false;
                };
                let option = path.path.is_ident("Some")
                    && matches!(
                        self.context.indexes[ordinal].resolve_path(module, &path.path),
                        Resolution::Local
                    );
                let owning = self
                    .context
                    .resolve_external_call_paths(ordinal, module, &path.path)
                    .iter()
                    .any(|path| {
                        matches!(
                            path.format_path().as_str(),
                            "std::boxed::Box::new"
                                | "alloc::boxed::Box::new"
                                | "std::sync::Arc::new"
                                | "alloc::sync::Arc::new"
                                | "std::rc::Rc::new"
                                | "alloc::rc::Rc::new"
                                | "std::option::Option::Some"
                                | "core::option::Option::Some"
                        )
                    });
                (option || owning)
                    && call
                        .args
                        .iter()
                        .any(|argument| self.carries_parameter(ordinal, module, argument, name))
            }
            syn::Expr::Paren(paren) => self.carries_parameter(ordinal, module, &paren.expr, name),
            syn::Expr::Reference(reference) => {
                self.carries_parameter(ordinal, module, &reference.expr, name)
            }
            _ => false,
        }
    }

    fn pattern_shadows(pattern: &syn::Pat, name: &syn::Ident) -> bool {
        match pattern {
            syn::Pat::Ident(binding) => {
                binding.ident == *name
                    || binding
                        .subpat
                        .as_ref()
                        .is_some_and(|(_, pattern)| Self::pattern_shadows(pattern, name))
            }
            syn::Pat::Type(pattern) => Self::pattern_shadows(&pattern.pat, name),
            syn::Pat::Paren(pattern) => Self::pattern_shadows(&pattern.pat, name),
            syn::Pat::Tuple(pattern) => pattern
                .elems
                .iter()
                .any(|pattern| Self::pattern_shadows(pattern, name)),
            syn::Pat::TupleStruct(pattern) => pattern
                .elems
                .iter()
                .any(|pattern| Self::pattern_shadows(pattern, name)),
            syn::Pat::Struct(pattern) => pattern
                .fields
                .iter()
                .any(|field| Self::pattern_shadows(&field.pat, name)),
            _ => false,
        }
    }

    fn shadows_parameter(block: &syn::Block, name: &syn::Ident) -> bool {
        block.stmts.iter().any(|statement| matches!(statement, syn::Stmt::Local(local) if Self::pattern_shadows(&local.pat, name)))
    }

    fn stores_callback(
        &self,
        ordinal: usize,
        module: usize,
        expression: &syn::Expr,
        name: &syn::Ident,
        escaping: bool,
        borrowed_receiver: bool,
    ) -> bool {
        match expression {
            syn::Expr::Assign(assignment) => {
                borrowed_receiver
                    && matches!(&*assignment.left, syn::Expr::Field(field) if matches!(&*field.base, syn::Expr::Path(path) if path.path.is_ident("self")))
                    && self.carries_parameter(ordinal, module, &assignment.right, name)
            }
            syn::Expr::Struct(structure) => {
                escaping
                    && structure
                        .fields
                        .iter()
                        .any(|field| self.carries_parameter(ordinal, module, &field.expr, name))
            }
            syn::Expr::Return(returned) => returned.expr.as_ref().is_some_and(|value| {
                self.stores_callback(ordinal, module, value, name, true, borrowed_receiver)
            }),
            syn::Expr::Block(block) => {
                !Self::shadows_parameter(&block.block, name)
                    && matches!(block.block.stmts.last(), Some(syn::Stmt::Expr(expression, semicolon)) if self.stores_callback(ordinal, module, expression, name, escaping && semicolon.is_none(), borrowed_receiver))
            }
            _ => false,
        }
    }

    fn callback_parameter(
        &self,
        ordinal: usize,
        module: usize,
        signature: &syn::Signature,
        position: usize,
    ) -> bool {
        let Some(syn::FnArg::Typed(parameter)) = signature
            .inputs
            .iter()
            .filter(|argument| !matches!(argument, syn::FnArg::Receiver(_)))
            .nth(position)
        else {
            return false;
        };
        match &*parameter.ty {
            syn::Type::ImplTrait(typed) => typed.bounds.iter().any(|bound| self.callable_bound(ordinal, module, bound)),
            syn::Type::Path(typed) => signature.generics.type_params().any(|generic| typed.path.is_ident(&generic.ident) && generic.bounds.iter().any(|bound| self.callable_bound(ordinal, module, bound)))
                || signature.generics.where_clause.as_ref().is_some_and(|clause| clause.predicates.iter().any(|predicate| matches!(predicate, syn::WherePredicate::Type(predicate) if matches!(&predicate.bounded_ty, syn::Type::Path(path) if path.path.get_ident() == typed.path.get_ident()) && predicate.bounds.iter().any(|bound| self.callable_bound(ordinal, module, bound))))),
            _ => false,
        }
    }

    fn registered_parameter(
        &self,
        ordinal: usize,
        module: usize,
        signature: &syn::Signature,
        block: &syn::Block,
        position: usize,
    ) -> bool {
        if !self.callback_parameter(ordinal, module, signature, position) {
            return false;
        }
        let Some(syn::FnArg::Typed(parameter)) = signature
            .inputs
            .iter()
            .filter(|argument| !matches!(argument, syn::FnArg::Receiver(_)))
            .nth(position)
        else {
            return false;
        };
        let Some(name) = Self::pattern_name(&parameter.pat) else {
            return false;
        };
        let borrowed_receiver = signature.receiver().is_some_and(|receiver| matches!(&receiver.kind, syn::ReceiverKind::Reference(..)) || matches!(&receiver.kind, syn::ReceiverKind::Typed(_, typed) if matches!(&**typed, syn::Type::Reference(_))));
        !Self::shadows_parameter(block, name)
            && matches!(block.stmts.last(), Some(syn::Stmt::Expr(expression, semicolon)) if self.stores_callback(ordinal, module, expression, name, semicolon.is_none(), borrowed_receiver))
    }

    fn call_registers_callback(&self, call: &syn::ExprCall, position: usize) -> bool {
        let syn::Expr::Path(path) = &*call.func else {
            return false;
        };
        if self
            .bindings
            .iter()
            .rev()
            .any(|binding| path.path.is_ident(&binding.name))
        {
            return false;
        }
        let active = node_predicate(
            &self.context.indexes[self.ordinal],
            self.module,
            &self.file.path,
            find_node_span(call.span()),
        );
        self.context.resolve_workspace_path(self.ordinal, self.module, &path.path).iter().any(|target| {
            let Some(identity) = target.path.item else { return false; };
            let item = &self.context.indexes[target.index].items[identity];
            active.and(&target.path.cfg).and(&item.cfg).is_satisfiable() && matches!(&item.syntax, syn::Item::Fn(function) if self.registered_parameter(target.index, item.module, &function.sig, &function.block, position))
        })
    }

    fn method_registers_callback(&self, call: &syn::ExprMethodCall, position: usize) -> bool {
        let Some(typed) = self.receiver_type(&call.receiver) else {
            return false;
        };
        let active = node_predicate(
            &self.context.indexes[self.ordinal],
            self.module,
            &self.file.path,
            find_node_span(call.span()),
        );
        self.context.resolve_workspace_type(self.ordinal, self.module, &typed).iter().any(|target| {
            let Some(identity) = target.path.item else { return false; };
            let index = &self.context.indexes[target.index];
            index.implementations.iter().filter(|implementation| active.and(&target.path.cfg).and(&implementation.cfg).is_satisfiable() && index.resolve_type_items(implementation.module, &implementation.syntax.self_ty).contains(&identity)).any(|implementation| implementation.syntax.items.iter().any(|item| matches!(item, syn::ImplItem::Fn(function) if function.sig.ident == call.method && active.and(&implementation.cfg).and(&crate::source::model::CfgPredicate::from_attributes(&function.attrs)).is_satisfiable() && self.registered_parameter(target.index, implementation.module, &function.sig, &function.block, position))))
        })
    }

    fn visit_condition(&mut self, condition: &syn::Expr) {
        match condition {
            syn::Expr::Let(condition) => {
                self.visit_expr(&condition.expr);
                self.shadow_pattern(&condition.pat);
            }
            syn::Expr::Binary(binary) if matches!(binary.op, syn::BinOp::And(_)) => {
                self.visit_condition(&binary.left);
                self.visit_condition(&binary.right);
            }
            _ => self.visit_expr(condition),
        }
    }

    fn visit_callback_argument(&mut self, argument: &syn::Expr) {
        let bound = if let syn::Expr::Path(path) = argument {
            self.closures
                .iter()
                .rev()
                .find(|(name, _, _)| path.path.is_ident(name))
                .and_then(|(_, closure, environment)| {
                    closure.clone().map(|closure| (closure, *environment))
                })
        } else {
            None
        };
        if let Some((closure, environment)) = bound {
            let later_bindings = self.closures.split_off(environment);
            let later_types = self.bindings.split_off(environment);
            self.callback_depth += 1;
            self.visit_expr_closure(&closure);
            self.callback_depth -= 1;
            self.closures.truncate(environment);
            self.closures.extend(later_bindings);
            self.bindings.truncate(environment);
            self.bindings.extend(later_types);
        }
    }

    fn consume_argument(&mut self, argument: &syn::Expr, is_drop: bool) {
        if self.rule != "DetachedTask" {
            return;
        }
        if is_drop {
            let spans = self
                .resolve_handles(argument)
                .into_iter()
                .filter_map(|identity| {
                    let (_, span, consumed) = self.handles[identity];
                    (!consumed).then_some(span)
                })
                .collect::<Vec<_>>();
            for span in spans {
                self.report(span);
            }
        }
        self.consume_handle(argument);
    }

    fn call_violation(&self, canonical: Option<&str>) -> bool {
        let spawn = canonical.is_some_and(Self::is_spawn_path);
        let detached = spawn
            || canonical.is_some_and(Self::is_blocking_spawn_path)
            || canonical.is_some_and(Self::is_local_spawn_path);
        match self.rule {
            "ThreadPrimitive" => {
                canonical == Some("std::thread::spawn")
                    && (self.discard && self.is_technology_module()
                        || self.thread_expect && !self.is_technology_module())
            }
            "DetachedTask" => detached && self.discard,
            "BridgeCleanup" => {
                (spawn || canonical == Some("std::thread::sleep")) && self.drop_context
            }
            "CallbackBridgeShape" => detached && self.callback_depth > 0,
            "BlockingCallInAsync" => {
                self.async_context
                    && canonical.is_some_and(|path| {
                        path == "std::thread::sleep"
                            || path.starts_with("std::fs::")
                            || path.starts_with("std::sync::mpsc::")
                    })
            }
            _ => false,
        }
    }
}

impl<'ast> Visit<'ast> for TaskVisitor<'_> {
    fn visit_fn_arg(&mut self, argument: &'ast syn::FnArg) {
        let syn::FnArg::Typed(parameter) = argument else {
            return;
        };
        self.shadow_pattern(&parameter.pat);
        if let Some(name) = Self::pattern_name(&parameter.pat)
            && let Some(binding) = self
                .bindings
                .iter_mut()
                .rev()
                .find(|binding| *name == binding.name)
        {
            binding.typed = Some((*parameter.ty).clone());
        }
    }
    fn visit_pat_ident(&mut self, pattern: &'ast syn::PatIdent) {
        self.shadow_pattern(&syn::Pat::Ident(pattern.clone()));
    }
    fn visit_expr_if(&mut self, expression: &'ast syn::ExprIf) {
        let bindings = self.bindings.len();
        let closures = self.closures.len();
        self.visit_condition(&expression.cond);
        if let syn::Expr::Lit(literal) = &*expression.cond
            && let syn::Lit::Bool(value) = &literal.lit
        {
            if value.value {
                self.visit_block(&expression.then_branch);
            } else if let Some((_, otherwise)) = &expression.else_branch {
                self.visit_expr(otherwise);
            }
            self.bindings.truncate(bindings);
            self.closures.truncate(closures);
            return;
        }
        let handles = self.handles.clone();
        self.visit_block(&expression.then_branch);
        let then_handles = self.handles.clone();
        self.handles = handles;
        self.bindings.truncate(bindings);
        self.closures.truncate(closures);
        if let Some((_, otherwise)) = &expression.else_branch {
            self.visit_expr(otherwise);
        }
        for (handle, then_handle) in self.handles.iter_mut().zip(then_handles) {
            handle.2 &= then_handle.2;
        }
    }
    fn visit_block(&mut self, block: &'ast syn::Block) {
        let closures = self.closures.len();
        let bindings = self.bindings.len();
        let length = self.handles.len();
        let capture = self
            .loop_prefix
            .as_ref()
            .filter(|loop_prefix| loop_prefix.body == std::ptr::from_ref(block) as usize)
            .map(|loop_prefix| loop_prefix.prefix);
        if let Some(prefix) = capture {
            for (position, statement) in block.stmts.iter().enumerate() {
                if position == prefix {
                    self.record_loop_consumption();
                }
                self.visit_stmt(statement);
            }
            if prefix == block.stmts.len() {
                self.record_loop_consumption();
            }
        } else {
            syn::visit::visit_block(self, block);
        }
        if let Some(syn::Stmt::Expr(value, None)) = block.stmts.last() {
            self.consume_handle(value);
        }
        let abandoned = self
            .handles
            .drain(length..)
            .filter_map(|(_, span, consumed)| (!consumed).then_some(span))
            .collect::<Vec<_>>();
        if self.rule == "DetachedTask" {
            for span in abandoned {
                self.report(span);
            }
        }
        self.closures.truncate(closures);
        self.bindings.truncate(bindings);
    }

    fn visit_expr_for_loop(&mut self, expression: &'ast syn::ExprForLoop) {
        self.visit_expr(&expression.expr);
        let handle_pattern = Self::pattern_name(&expression.pat).is_some();
        let inspect_order = self.rule == "DetachedTask" && handle_pattern;
        let identities = inspect_order
            .then(|| self.resolve_ordered_handles(&expression.expr))
            .flatten();
        if let Some(identities) = identities {
            self.visit_ordered_array(expression, identities);
            return;
        }
        let bindings = self.bindings.len();
        let closures = self.closures.len();
        let handles = self.handles.clone();
        self.shadow_pattern(&expression.pat);
        self.visit_block(&expression.body);
        self.bindings.truncate(bindings);
        self.closures.truncate(closures);
        self.handles = handles;
    }

    fn visit_expr_while(&mut self, expression: &'ast syn::ExprWhile) {
        let bindings = self.bindings.len();
        let closures = self.closures.len();
        let handles = self.handles.clone();
        self.visit_condition(&expression.cond);
        self.visit_block(&expression.body);
        self.bindings.truncate(bindings);
        self.closures.truncate(closures);
        self.handles = handles;
    }

    fn visit_expr_match(&mut self, expression: &'ast syn::ExprMatch) {
        self.visit_expr(&expression.expr);
        let handles = self.handles.clone();
        let mut consumed = vec![true; handles.len()];
        for arm in &expression.arms {
            self.handles = handles.clone();
            self.visit_arm(arm);
            for (consumed, handle) in consumed.iter_mut().zip(&self.handles) {
                *consumed &= handle.2;
            }
        }
        self.handles = handles;
        for (handle, consumed) in self.handles.iter_mut().zip(consumed) {
            handle.2 = consumed;
        }
    }

    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        let bindings = self.bindings.len();
        let closures = self.closures.len();
        self.shadow_pattern(&arm.pat);
        self.visit_expr(&arm.body);
        self.bindings.truncate(bindings);
        self.closures.truncate(closures);
    }

    fn visit_expr_await(&mut self, expression: &'ast syn::ExprAwait) {
        self.consume_handle(&expression.base);
        syn::visit::visit_expr_await(self, expression);
    }

    fn visit_expr_return(&mut self, expression: &'ast syn::ExprReturn) {
        if let Some(value) = &expression.expr {
            self.consume_handle(value);
        }
        let discard = self.discard;
        self.discard = false;
        syn::visit::visit_expr_return(self, expression);
        self.discard = discard;
        if self.rule == "DetachedTask" {
            let pending = self
                .handles
                .iter()
                .skip(self.return_start)
                .filter_map(|(_, span, consumed)| (!consumed).then_some(*span))
                .collect::<Vec<_>>();
            for span in pending {
                self.report(span);
            }
        }
    }

    fn visit_expr_struct(&mut self, expression: &'ast syn::ExprStruct) {
        for field in &expression.fields {
            self.consume_handle(&field.expr);
        }
        syn::visit::visit_expr_struct(self, expression);
    }
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        let expectation = self.thread_expect;
        self.thread_expect |= expects_lint(&item.attrs, "disallowed_methods");
        let parent = self.module;
        if let Some(module) = self.context.indexes[self.ordinal]
            .modules
            .iter()
            .find(|module| {
                module.parent == Some(parent) && module.span == find_node_span(item.span())
            })
        {
            self.module = module.id;
        }
        syn::visit::visit_item_mod(self, item);
        self.module = parent;
        self.thread_expect = expectation;
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        let return_start = self.return_start;
        self.return_start = 0;
        let handles = std::mem::take(&mut self.handles);
        let closures = std::mem::take(&mut self.closures);
        let bindings = std::mem::take(&mut self.bindings);
        let expectation = self.thread_expect;
        self.thread_expect |= expects_lint(&item.attrs, "disallowed_methods");
        let parent = (self.async_context, self.drop_context, self.callback_depth);
        let test_function = self.test_function;
        self.test_function = item.attrs.iter().any(Self::is_test_attribute);
        self.async_context = item.sig.asyncness.is_some();
        self.drop_context = false;
        self.callback_depth = 0;
        syn::visit::visit_item_fn(self, item);
        self.test_function = test_function;
        (self.async_context, self.drop_context, self.callback_depth) = parent;
        self.thread_expect = expectation;
        self.handles = handles;
        self.closures = closures;
        self.bindings = bindings;
        self.return_start = return_start;
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let self_type = self.self_type.replace((*item.self_ty).clone());
        let local_self = self.local_self;
        self.local_self = !self.context.indexes[self.ordinal]
            .resolve_type_items(self.module, &item.self_ty)
            .is_empty();
        let expectation = self.thread_expect;
        self.thread_expect |= expects_lint(&item.attrs, "disallowed_methods");
        let parent = self.drop_context;
        self.drop_context = item.trait_.as_ref().is_some_and(|(path, _)| {
            self.external(path).as_deref() == Some("std::ops::Drop")
                || self.external(path).as_deref() == Some("core::ops::Drop")
        });
        syn::visit::visit_item_impl(self, item);
        self.drop_context = parent;
        self.thread_expect = expectation;
        self.local_self = local_self;
        self.self_type = self_type;
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        let return_start = self.return_start;
        self.return_start = 0;
        let handles = std::mem::take(&mut self.handles);
        let bindings = std::mem::take(&mut self.bindings);
        let closures = std::mem::take(&mut self.closures);
        let callback = self.callback_depth;
        self.callback_depth = 0;
        let expectation = self.thread_expect;
        self.thread_expect |= expects_lint(&item.attrs, "disallowed_methods");
        let parent = self.async_context;
        self.async_context = item.sig.asyncness.is_some();
        let parent_drop = self.drop_context;
        self.drop_context &= item.sig.ident == "drop";
        syn::visit::visit_impl_item_fn(self, item);
        self.async_context = parent;
        self.drop_context = parent_drop;
        self.thread_expect = expectation;
        self.callback_depth = callback;
        self.handles = handles;
        self.bindings = bindings;
        self.closures = closures;
        self.return_start = return_start;
    }

    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        let parent = (
            self.async_context,
            self.drop_context,
            self.callback_depth,
            self.thread_expect,
        );
        self.async_context = item.sig.asyncness.is_some();
        self.drop_context = false;
        self.callback_depth = 0;
        self.thread_expect |= expects_lint(&item.attrs, "disallowed_methods");
        syn::visit::visit_trait_item_fn(self, item);
        (
            self.async_context,
            self.drop_context,
            self.callback_depth,
            self.thread_expect,
        ) = parent;
    }

    fn visit_expr_async(&mut self, expression: &'ast syn::ExprAsync) {
        let parent = self.async_context;
        let return_start = self.return_start;
        self.return_start = self.handles.len();
        self.async_context = true;
        syn::visit::visit_expr_async(self, expression);
        self.async_context = parent;
        self.return_start = return_start;
    }

    fn visit_expr_closure(&mut self, expression: &'ast syn::ExprClosure) {
        let return_start = self.return_start;
        self.return_start = self.handles.len();
        let bindings = self.bindings.len();
        let closures = self.closures.len();
        for parameter in &expression.inputs {
            self.shadow_pattern(parameter);
        }
        let parent = self.async_context;
        let inherited = std::mem::take(&mut self.inherit_async);
        self.async_context = expression.asyncness.is_some() || inherited;
        syn::visit::visit_expr_closure(self, expression);
        self.async_context = parent;
        self.bindings.truncate(bindings);
        self.closures.truncate(closures);
        self.return_start = return_start;
    }

    fn visit_expr(&mut self, expression: &'ast syn::Expr) {
        let discard = self.discard;
        if self.rule == "ThreadPrimitive" && !Self::is_discard_transparent(expression) {
            self.discard = false;
        }
        syn::visit::visit_expr(self, expression);
        self.discard = discard;
    }

    fn visit_stmt(&mut self, statement: &'ast syn::Stmt) {
        let parent = self.discard;
        self.discard = matches!(statement, syn::Stmt::Expr(_, Some(_)));
        syn::visit::visit_stmt(self, statement);
        self.discard = parent;
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        let expectation = self.thread_expect;
        self.thread_expect |= expects_lint(&local.attrs, "disallowed_methods");
        let parent = self.discard;
        self.discard = matches!(local.pat, syn::Pat::Wild(_));
        if let Some(init) = &local.init {
            if self.rule == "CallbackBridgeShape"
                && self.discard
                && self.discarded_guard(&init.expr)
            {
                self.report(local.span());
            }
            self.visit_expr(&init.expr);
            if let Some((_, diverge)) = &init.diverge {
                self.visit_expr(diverge);
            }
        }
        self.discard = parent;
        self.thread_expect = expectation;
        self.bind_local(local);
    }

    fn visit_expr_assign(&mut self, expression: &'ast syn::ExprAssign) {
        let discard = self.discard;
        self.discard = self.discard
            && (self.rule != "ThreadPrimitive" || Self::is_underscore_assignment(expression));
        syn::visit::visit_expr_assign(self, expression);
        self.discard = discard;
    }

    fn visit_expr_call(&mut self, expression: &'ast syn::ExprCall) {
        let expectation = self.thread_expect;
        self.thread_expect |= expects_lint(&expression.attrs, "disallowed_methods");
        let canonical = match &*expression.func {
            syn::Expr::Path(path) => self.external(&path.path),
            _ => None,
        };
        if self.call_violation(canonical.as_deref()) {
            self.report(expression.span());
        }
        self.check_unwind(expression, canonical.as_deref());
        let is_drop = canonical
            .as_deref()
            .is_some_and(|path| matches!(path, "std::mem::drop" | "core::mem::drop"));
        let inherit_async =
            self.async_context && !canonical.as_deref().is_some_and(Self::runs_off_async);
        for (position, argument) in expression.args.iter().enumerate() {
            let registered = self.rule == "CallbackBridgeShape"
                && (self.call_registers_callback(expression, position)
                    || self.boxed_to_foreign_function(expression, argument));
            if registered {
                self.visit_callback_argument(argument);
            }
            if self.rule != "DetachedTask" || !self.is_owning_call(expression) {
                self.consume_argument(argument, is_drop);
            }
            if self.is_dropped_thread_spawn(is_drop, argument) {
                self.report(argument.span());
            }
            if self.rule == "DetachedTask" && is_drop && self.is_spawn(argument) {
                self.report(argument.span());
                let parent = self.discard;
                self.discard = false;
                self.visit_expr(argument);
                self.discard = parent;
            } else {
                let callback = registered
                    && (matches!(argument, syn::Expr::Closure(_))
                        || self.is_boxed_closure(argument));
                self.callback_depth += usize::from(callback);
                let parent = self.discard;
                self.discard = false;
                self.inherit_async = inherit_async && matches!(argument, syn::Expr::Closure(_));
                self.visit_expr(argument);
                self.inherit_async = false;
                self.discard = parent;
                self.callback_depth -= usize::from(callback);
            }
        }
        self.visit_expr(&expression.func);
        self.thread_expect = expectation;
    }

    fn visit_expr_method_call(&mut self, expression: &'ast syn::ExprMethodCall) {
        if self.rule == "CallbackBridgeShape"
            && self.callback_depth > 0
            && expression.method == "blocking_send"
            && self
                .receiver_type(&expression.receiver)
                .is_some_and(|typed| {
                    self.context
                        .resolve_outer_external_type_paths(self.ordinal, self.module, &typed)
                        .iter()
                        .any(|path| path.format_path() == "tokio::sync::mpsc::Sender")
                })
        {
            self.report(expression.span());
        }
        if self.rule == "DetachedTask" && self.discard && self.is_runtime_spawn(expression) {
            self.report(expression.span());
        }
        if self.rule == "CallbackBridgeShape"
            && self.callback_depth > 0
            && self.is_runtime_spawn(expression)
        {
            self.report(expression.span());
        }
        if self.is_block_on_in_drop(expression) {
            self.report(expression.span());
        }
        if self.is_discarded_builder_spawn(expression) {
            self.report(expression.span());
        }
        if expression.method == "abort" && !self.test_function {
            self.consume_handle(&expression.receiver);
        }
        let discard = self.discard;
        self.discard = self.rule == "ThreadPrimitive"
            && discard
            && matches!(expression.method.to_string().as_str(), "unwrap" | "expect");
        self.visit_expr(&expression.receiver);
        self.discard = false;
        let inherit_async = self.async_context
            && !matches!(
                expression.method.to_string().as_str(),
                "spawn" | "spawn_blocking" | "spawn_scoped"
            );
        for (position, argument) in expression.args.iter().enumerate() {
            let registered = self.rule == "CallbackBridgeShape"
                && (self.method_registers_callback(expression, position)
                    || self.boxed_to_foreign_method(expression, argument));
            if registered {
                self.visit_callback_argument(argument);
            }
            self.consume_handle(argument);
            let callback = registered
                && (matches!(argument, syn::Expr::Closure(_)) || self.is_boxed_closure(argument));
            self.callback_depth += usize::from(callback);
            self.inherit_async = inherit_async && matches!(argument, syn::Expr::Closure(_));
            self.visit_expr(argument);
            self.inherit_async = false;
            self.callback_depth -= usize::from(callback);
        }
        self.discard = discard;
    }

    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        for argument in &parse_macro_expressions(item) {
            self.visit_expr(argument);
        }
    }
}

struct LocalBinding {
    name: String,
    handles: Vec<usize>,
    typed: Option<syn::Type>,
    is_ordered: bool,
}

struct LoopPrefix {
    body: usize,
    prefix: usize,
    consumed: Vec<bool>,
}
