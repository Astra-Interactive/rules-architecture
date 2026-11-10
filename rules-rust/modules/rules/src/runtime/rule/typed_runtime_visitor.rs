use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::{build_node_finding, format_module_path};
use crate::runtime::rule::AwaitVisitor;
use crate::runtime::rule::ClosureEffects;
use crate::runtime::rule::GuardWrites;
use crate::runtime::rule::SelectBranches;
use crate::runtime::rule::TaskVisitor;
use crate::runtime::util::expects_lint;
use crate::runtime::util::node_predicate;
use crate::source::model::ResolvedTypeSyntax;
use crate::source::model::SourceFile;
use crate::source::util::find_node_span;
use std::rc::Rc;
use syn::spanned::Spanned;
use syn::visit::Visit;

const RESULT_TYPES: &[&str] = &["std::result::Result", "core::result::Result"];

const OPTION_TYPES: &[&str] = &["std::option::Option", "core::option::Option"];

const LOCK_TYPES: &[&str] = &[
    "tokio::sync::Mutex",
    "parking_lot::Mutex",
    "std::sync::Mutex",
    "tokio::sync::RwLock",
    "parking_lot::RwLock",
    "std::sync::RwLock",
];

const ATOMIC_TYPES: &[&str] = &[
    "std::sync::atomic::AtomicBool",
    "std::sync::atomic::AtomicU8",
    "std::sync::atomic::AtomicU16",
    "std::sync::atomic::AtomicU32",
    "std::sync::atomic::AtomicU64",
    "std::sync::atomic::AtomicUsize",
    "std::sync::atomic::AtomicI8",
    "std::sync::atomic::AtomicI16",
    "std::sync::atomic::AtomicI32",
    "std::sync::atomic::AtomicI64",
    "std::sync::atomic::AtomicIsize",
    "std::sync::atomic::AtomicPtr",
];

const ATOMIC_WRITES: &[&str] = &[
    "store",
    "swap",
    "fetch_add",
    "fetch_sub",
    "fetch_and",
    "fetch_nand",
    "fetch_or",
    "fetch_xor",
    "fetch_max",
    "fetch_min",
    "fetch_update",
    "compare_exchange",
    "compare_exchange_weak",
];

pub(crate) struct TypedRuntimeVisitor<'source> {
    context: &'source RuleContext,
    ordinal: usize,
    module: usize,
    file: &'source SourceFile,
    rule: &'static str,
    findings: Vec<RuleFinding>,
    bindings: Vec<(String, Rc<ResolvedTypeSyntax>)>,
    self_type: Option<syn::Type>,
    synchronous_boundary: bool,
    loops: usize,
    thread_expect: bool,
    async_context: bool,
    runtime_main: bool,
    inherit_async: bool,
}

impl TypedRuntimeVisitor<'_> {
    pub(crate) fn check_context(context: &RuleContext, rule: &'static str) -> Vec<RuleFinding> {
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
                let mut visitor = TypedRuntimeVisitor {
                    context,
                    ordinal,
                    module: module.id,
                    file,
                    rule,
                    findings: Vec::new(),
                    bindings: Vec::new(),
                    self_type: None,
                    synchronous_boundary: false,
                    loops: 0,
                    thread_expect: expects_lint(&file.syntax.attrs, "disallowed_methods"),
                    async_context: false,
                    runtime_main: false,
                    inherit_async: false,
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
        self.findings.push(build_node_finding(
            self.rule,
            &self.file.path,
            find_node_span(span),
            format_module_path(&index.crate_name, &index.modules[self.module].module.path),
            if self.rule == "ResultConsumption" {
                "The discard expectation requires a proven oneshot::Sender::send, mpsc::UnboundedSender::send, broadcast::Sender::send or mpsc::Sender::try_send; an unresolved receiver cannot establish this exception (error-handling.rs.md)".to_owned()
            } else { format!("Follow the {} typed runtime requirement (async.rs.md; time-api.rs.md)", self.rule) },
        ));
    }

    fn build_receiver_type(&self, typed: syn::Type, module: usize) -> Rc<ResolvedTypeSyntax> {
        Rc::new(ResolvedTypeSyntax {
            index: self.ordinal,
            module,
            typed,
            cfg: crate::source::model::CfgPredicate::All(Vec::new()),
            has_unsubstituted_parameters: false,
        })
    }

    fn wrapped_types(&self, expression: &syn::Expr, names: &[&str]) -> Vec<Rc<ResolvedTypeSyntax>> {
        self.receiver_types(expression)
            .into_iter()
            .flat_map(|receiver| {
                self.context.resolve_type_syntax_under(
                    receiver.index,
                    receiver.module,
                    &receiver.typed,
                    &receiver.cfg,
                )
            })
            .filter_map(|resolved| {
                let outer = self.context.resolve_outer_external_type_paths(
                    resolved.index,
                    resolved.module,
                    &resolved.typed,
                );
                if !outer
                    .iter()
                    .any(|path| names.contains(&path.format_path().as_str()))
                {
                    return None;
                }
                let syn::Type::Path(path) = &resolved.typed else {
                    return None;
                };
                let syn::PathArguments::AngleBracketed(arguments) =
                    &path.path.segments.last()?.arguments
                else {
                    return None;
                };
                let syn::GenericArgument::Type(success) = arguments.args.first()? else {
                    return None;
                };
                Some(Rc::new(ResolvedTypeSyntax {
                    typed: success.clone(),
                    ..resolved
                }))
            })
            .collect()
    }

    fn result_types(&self, expression: &syn::Expr) -> Vec<Rc<ResolvedTypeSyntax>> {
        self.wrapped_types(expression, RESULT_TYPES)
    }

    fn channel_method_type(&self, call: &syn::ExprMethodCall) -> Vec<Rc<ResolvedTypeSyntax>> {
        if matches!(call.method.to_string().as_str(), "lock" | "blocking_lock") {
            let guarded = self.wrapped_types(&call.receiver, LOCK_TYPES);
            if !guarded.is_empty() {
                return guarded;
            }
        }
        if call.method == "take" && !self.wrapped_types(&call.receiver, OPTION_TYPES).is_empty() {
            return self.receiver_types(&call.receiver);
        }
        if call.method == "subscribe" {
            if self.receiver_has(&call.receiver, &["tokio::sync::broadcast::Sender"]) {
                return vec![self.build_receiver_type(
                    syn::parse_quote!(tokio::sync::broadcast::Receiver),
                    self.module,
                )];
            }
            if self.receiver_has(&call.receiver, &["tokio::sync::watch::Sender"]) {
                return vec![self.build_receiver_type(
                    syn::parse_quote!(tokio::sync::watch::Receiver),
                    self.module,
                )];
            }
        }
        Vec::new()
    }

    fn runtime_method_type(&self, call: &syn::ExprMethodCall) -> Vec<Rc<ResolvedTypeSyntax>> {
        let channel = self.channel_method_type(call);
        if !channel.is_empty() {
            return channel;
        }
        if ["unwrap", "expect"].iter().any(|name| call.method == *name) {
            let success = self.result_types(&call.receiver);
            if !success.is_empty() {
                return success;
            }
            return self.receiver_types(&call.receiver);
        }
        if call.method == "map_err" && !self.result_types(&call.receiver).is_empty() {
            return self.receiver_types(&call.receiver);
        }
        if call.method == "clone" {
            return self.receiver_types(&call.receiver);
        }
        if call.method == "build" && self.receiver_has(&call.receiver, &["tokio::runtime::Builder"])
        {
            return vec![self.build_receiver_type(
                syn::parse_quote!(::std::result::Result<tokio::runtime::Runtime, _>),
                self.module,
            )];
        }
        if matches!(
            call.method.to_string().as_str(),
            "enable_all"
                | "enable_io"
                | "enable_time"
                | "worker_threads"
                | "max_blocking_threads"
                | "thread_name"
                | "thread_stack_size"
        ) && self.receiver_has(&call.receiver, &["tokio::runtime::Builder"])
        {
            return self.receiver_types(&call.receiver);
        }
        if call.method == "handle"
            && self.receiver_has(&call.receiver, &["tokio::runtime::Runtime"])
        {
            return vec![
                self.build_receiver_type(syn::parse_quote!(tokio::runtime::Handle), self.module),
            ];
        }
        Vec::new()
    }

    fn tail_expression(expression: &syn::Expr) -> &syn::Expr {
        if let syn::Expr::Block(block) = expression
            && let [syn::Stmt::Expr(tail, None)] = block.block.stmts.as_slice()
        {
            return Self::tail_expression(tail);
        }
        expression
    }

    fn diverges(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Return(_) | syn::Expr::Break(_) | syn::Expr::Continue(_) => true,
            syn::Expr::Block(block) => block.block.stmts.iter().any(|statement| matches!(statement, syn::Stmt::Expr(expression, _) if self.diverges(expression))),
            syn::Expr::Macro(expression) => self.context.resolve_external_call_paths(self.ordinal, self.module, &expression.mac.path).iter().any(|path| matches!(path.format_path().as_str(), "std::panic" | "core::panic" | "std::unreachable" | "core::unreachable")),
            syn::Expr::Call(call) => matches!(&*call.func, syn::Expr::Path(path) if self.context.resolve_external_call_paths(self.ordinal, self.module, &path.path).iter().any(|path| matches!(path.format_path().as_str(), "std::process::exit" | "std::process::abort"))),
            _ => false,
        }
    }

    fn match_result_types(&self, expression: &syn::ExprMatch) -> Vec<Rc<ResolvedTypeSyntax>> {
        let success = self.result_types(&expression.expr);
        if success.is_empty() {
            return Vec::new();
        }
        let mut returns_success = false;
        for arm in &expression.arms {
            if self.diverges(&arm.body) {
                continue;
            }
            let syn::Pat::TupleStruct(pattern) = &arm.pat else {
                return Vec::new();
            };
            if !crate::runtime::util::is_result_variant(
                self.context,
                self.ordinal,
                self.module,
                &pattern.path,
                "Ok",
            ) {
                return Vec::new();
            }
            let Some(syn::Pat::Ident(binding)) = pattern.elems.first() else {
                return Vec::new();
            };
            if !matches!(Self::tail_expression(&arm.body), syn::Expr::Path(path) if path.path.is_ident(&binding.ident))
            {
                return Vec::new();
            }
            returns_success = true;
        }
        if returns_success { success } else { Vec::new() }
    }

    fn self_field_receiver_types(&self, field: &syn::ExprField) -> Vec<Rc<ResolvedTypeSyntax>> {
        let Some(typed) = &self.self_type else {
            return Vec::new();
        };
        self.context.indexes[self.ordinal]
            .resolve_type_items(self.module, typed)
            .iter()
            .flat_map(|identity| {
                let declaration = &self.context.indexes[self.ordinal].items[*identity];
                let syn::Item::Struct(item) = &declaration.syntax else {
                    return Vec::new();
                };
                item.fields
                    .iter()
                    .enumerate()
                    .filter_map(|(position, entry)| {
                        let matching = match &field.member {
                            syn::Member::Named(name) => entry.ident.as_ref() == Some(name),
                            syn::Member::Unnamed(index) => position == index.index as usize,
                        };
                        matching
                            .then(|| self.build_receiver_type(entry.ty.clone(), declaration.module))
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn awaited_types(&self, awaited: &syn::ExprAwait) -> Vec<Rc<ResolvedTypeSyntax>> {
        let syn::Expr::MethodCall(call) = &*awaited.base else {
            return Vec::new();
        };
        if call.method != "recv"
            || !self.receiver_has(&call.receiver, &["tokio::sync::broadcast::Receiver"])
        {
            return Vec::new();
        }
        vec![self.build_receiver_type(
            syn::parse_quote!(::std::result::Result<_, tokio::sync::broadcast::error::RecvError>),
            self.module,
        )]
    }

    fn receiver_types(&self, expression: &syn::Expr) -> Vec<Rc<ResolvedTypeSyntax>> {
        match expression {
            syn::Expr::Reference(reference) => self.receiver_types(&reference.expr),
            syn::Expr::Paren(paren) => self.receiver_types(&paren.expr),
            syn::Expr::Group(group) => self.receiver_types(&group.expr),
            syn::Expr::Try(expression) => self.result_types(&expression.expr),
            syn::Expr::Match(expression) => self.match_result_types(expression),
            syn::Expr::Call(_) => self
                .constructor_type(expression)
                .map(|typed| vec![self.build_receiver_type(typed, self.module)])
                .unwrap_or_default(),
            syn::Expr::MethodCall(call) => self.runtime_method_type(call),
            syn::Expr::Await(awaited) => self.awaited_types(awaited),
            syn::Expr::Path(path) => self
                .bindings
                .iter()
                .rev()
                .find(|(name, _)| path.path.is_ident(name))
                .map(|(_, typed)| vec![Rc::clone(typed)])
                .unwrap_or_default(),
            syn::Expr::Field(field) if matches!(&*field.base, syn::Expr::Path(path) if path.path.is_ident("self")) => {
                self.self_field_receiver_types(field)
            }
            _ => Vec::new(),
        }
    }

    fn receiver_has(&self, expression: &syn::Expr, names: &[&str]) -> bool {
        let active = node_predicate(
            &self.context.indexes[self.ordinal],
            self.module,
            &self.file.path,
            find_node_span(expression.span()),
        );
        self.receiver_types(expression).iter().any(|receiver| {
            let compatible = active
                .in_scope(self.ordinal)
                .and(&receiver.cfg.in_scope(receiver.index));
            compatible.is_satisfiable()
                && !receiver.has_unsubstituted_parameters
                && self.receiver_identity(
                    receiver.index,
                    receiver.module,
                    &receiver.typed,
                    &compatible,
                    names,
                    0,
                )
        })
    }

    fn receiver_identity(
        &self,
        ordinal: usize,
        module: usize,
        typed: &syn::Type,
        active: &crate::source::model::CfgPredicate,
        names: &[&str],
        depth: usize,
    ) -> bool {
        if depth > 8 {
            return false;
        }
        let inner = match typed {
            syn::Type::Reference(value) => Some(&*value.elem),
            syn::Type::Paren(value) => Some(&*value.elem),
            syn::Type::Group(value) => Some(&*value.elem),
            _ => None,
        };
        if let Some(inner) = inner {
            return self.receiver_identity(ordinal, module, inner, active, names, depth + 1);
        }
        self.context.resolve_type_syntax(ordinal, module, typed).iter().any(|resolved| {
            if !active.and(&resolved.cfg.in_scope(resolved.index)).is_satisfiable() { return false; }
            let outer = self.context.resolve_outer_external_type_paths(resolved.index, resolved.module, &resolved.typed);
            if outer.iter().any(|path| names.contains(&path.format_path().as_str())) { return true; }
            if !outer.iter().any(|path| matches!(path.format_path().as_str(), "std::sync::Arc" | "alloc::sync::Arc" | "std::rc::Rc" | "alloc::rc::Rc" | "std::boxed::Box" | "alloc::boxed::Box" | "std::pin::Pin" | "core::pin::Pin")) { return false; }
            let syn::Type::Path(path) = &resolved.typed else { return false; };
            let Some(segment) = path.path.segments.last() else { return false; };
            let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else { return false; };
            arguments.args.iter().any(|argument| matches!(argument, syn::GenericArgument::Type(inner)
                if self.receiver_identity(resolved.index, resolved.module, inner, active, names, depth + 1)))
        })
    }

    fn is_blocking_method(&self, expression: &syn::ExprMethodCall) -> bool {
        let blocking_channel = self.receiver_has(
            &expression.receiver,
            &[
                "std::sync::mpsc::Receiver",
                "std::sync::mpsc::Sender",
                "std::sync::mpsc::SyncSender",
            ],
        );
        let blocking_file_read = matches!(
            expression.method.to_string().as_str(),
            "read_to_end" | "read_to_string" | "read_exact"
        ) && self.receiver_has(&expression.receiver, &["std::fs::File"]);
        blocking_channel || blocking_file_read
    }

    fn bind_signature(&mut self, signature: &syn::Signature) {
        for argument in &signature.inputs {
            if let syn::FnArg::Typed(parameter) = argument
                && let syn::Pat::Ident(name) = &*parameter.pat
            {
                self.bindings.push((
                    name.ident.to_string(),
                    self.build_receiver_type((*parameter.ty).clone(), self.module),
                ));
            }
        }
    }

    fn bind_pattern(&mut self, pattern: &syn::Pat) {
        match pattern {
            syn::Pat::Ident(name) => self.bindings.push((
                name.ident.to_string(),
                self.build_receiver_type(syn::parse_quote!(_), self.module),
            )),
            syn::Pat::Type(pattern) => {
                if let syn::Pat::Ident(name) = &*pattern.pat {
                    self.bindings.push((
                        name.ident.to_string(),
                        self.build_receiver_type((*pattern.ty).clone(), self.module),
                    ));
                }
            }
            syn::Pat::Tuple(pattern) => {
                for element in &pattern.elems {
                    self.bind_pattern(element);
                }
            }
            syn::Pat::TupleStruct(pattern) => {
                for element in &pattern.elems {
                    self.bind_pattern(element);
                }
            }
            syn::Pat::Struct(pattern) => {
                for field in &pattern.fields {
                    self.bind_pattern(&field.pat);
                }
            }
            syn::Pat::Reference(pattern) => self.bind_pattern(&pattern.pat),
            syn::Pat::Paren(pattern) => self.bind_pattern(&pattern.pat),
            _ => {}
        }
    }

    fn constructor_type(&self, expression: &syn::Expr) -> Option<syn::Type> {
        let syn::Expr::Call(call) = expression else {
            return None;
        };
        let syn::Expr::Path(path) = &*call.func else {
            return None;
        };
        let method = path.path.segments.last()?;
        let constructor = [
            "new",
            "default",
            "with_capacity",
            "new_current_thread",
            "new_multi_thread",
            "current",
            "try_current",
        ]
        .iter()
        .any(|name| method.ident == name);
        if !constructor || path.path.segments.len() <= 1 {
            return None;
        }
        let mut owner = path.path.clone();
        owner.segments.pop();
        owner.segments.pop_punct();
        let typed = syn::Type::Path(syn::TypePath {
            attrs: Vec::new(),
            qself: None,
            path: owner,
        });
        let workspace =
            self.context
                .resolve_outer_workspace_type(self.ordinal, self.module, &typed);
        let external =
            self.context
                .resolve_outer_external_type_paths(self.ordinal, self.module, &typed);
        if workspace.is_empty() && external.is_empty() {
            return None;
        }
        let fallible = self
            .context
            .resolve_external_call_paths(self.ordinal, self.module, &path.path)
            .iter()
            .any(|path| {
                matches!(
                    path.format_path().as_str(),
                    "tokio::runtime::Runtime::new" | "tokio::runtime::Handle::try_current"
                )
            });
        if fallible {
            return Some(syn::parse_quote!(::std::result::Result<#typed, _>));
        }
        Some(typed)
    }

    fn channel_types(&self, path: &syn::Path) -> Option<(&'static str, &'static str)> {
        let canonical = self
            .context
            .resolve_external_call_paths(self.ordinal, self.module, path);
        canonical
            .iter()
            .find_map(|path| match path.format_path().as_str() {
                "tokio::sync::oneshot::channel" => Some((
                    "tokio::sync::oneshot::Sender",
                    "tokio::sync::oneshot::Receiver",
                )),
                "tokio::sync::mpsc::channel" => {
                    Some(("tokio::sync::mpsc::Sender", "tokio::sync::mpsc::Receiver"))
                }
                "tokio::sync::mpsc::unbounded_channel" => Some((
                    "tokio::sync::mpsc::UnboundedSender",
                    "tokio::sync::mpsc::UnboundedReceiver",
                )),
                "tokio::sync::broadcast::channel" => Some((
                    "tokio::sync::broadcast::Sender",
                    "tokio::sync::broadcast::Receiver",
                )),
                "tokio::sync::watch::channel" => {
                    Some(("tokio::sync::watch::Sender", "tokio::sync::watch::Receiver"))
                }
                _ => None,
            })
    }

    fn bind_channel(&mut self, pattern: &syn::PatTuple, sender: &str, receiver: &str) {
        for (pattern, typed) in pattern.elems.iter().zip([sender, receiver]) {
            if let syn::Pat::Ident(name) = pattern
                && let Ok(typed) = syn::parse_str::<syn::Type>(typed)
            {
                self.bindings.push((
                    name.ident.to_string(),
                    self.build_receiver_type(typed, self.module),
                ));
            }
        }
    }

    fn bind_payload(&mut self, pattern: &syn::Pat, expression: &syn::Expr) -> bool {
        let syn::Pat::TupleStruct(pattern) = pattern else {
            return false;
        };
        let Some(syn::Pat::Ident(name)) = pattern.elems.first() else {
            return false;
        };
        let types = if pattern.path.is_ident("Some") {
            self.wrapped_types(expression, OPTION_TYPES)
        } else if crate::runtime::util::is_result_variant(
            self.context,
            self.ordinal,
            self.module,
            &pattern.path,
            "Ok",
        ) {
            self.result_types(expression)
        } else {
            Vec::new()
        };
        let Some(typed) = types.into_iter().next() else {
            return false;
        };
        self.bindings.push((name.ident.to_string(), typed));
        true
    }

    fn bind_initializer(&mut self, local: &syn::Local) {
        let Some(init) = &local.init else {
            self.bind_pattern(&local.pat);
            return;
        };
        if let syn::Pat::Ident(name) = &local.pat {
            let mut types = self.receiver_types(&init.expr);
            if types.is_empty()
                && let Some(typed) = self.constructor_type(&init.expr)
            {
                types.push(self.build_receiver_type(typed, self.module));
            }
            if let Some(typed) = types.into_iter().next() {
                self.bindings.push((name.ident.to_string(), typed));
                return;
            }
        }
        if let syn::Pat::Tuple(pattern) = &local.pat
            && let syn::Expr::Call(call) = &*init.expr
            && let syn::Expr::Path(path) = &*call.func
            && let Some((sender, receiver)) = self.channel_types(&path.path)
        {
            self.bind_channel(pattern, sender, receiver);
            return;
        }
        if self.bind_payload(&local.pat, &init.expr) {
            return;
        }
        self.bind_pattern(&local.pat);
    }

    fn check_lock_type(&mut self, typed: &syn::Type) {
        let active = node_predicate(
            &self.context.indexes[self.ordinal],
            self.module,
            &self.file.path,
            find_node_span(typed.span()),
        );
        for resolved in self
            .context
            .resolve_type_syntax(self.ordinal, self.module, typed)
        {
            if resolved.has_unsubstituted_parameters || !active.and(&resolved.cfg).is_satisfiable()
            {
                continue;
            }
            let syn::Type::Path(path) = &resolved.typed else {
                continue;
            };
            let Some(segment) = path.path.segments.last() else {
                continue;
            };
            let outer = self.context.resolve_outer_external_type_paths(
                resolved.index,
                resolved.module,
                &resolved.typed,
            );
            if !outer.iter().any(|identity| {
                identity
                    .segments
                    .last()
                    .is_some_and(|name| matches!(name.as_str(), "Mutex" | "RwLock"))
            }) {
                continue;
            }
            let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                continue;
            };
            let Some(syn::GenericArgument::Type(inner)) = arguments.args.first() else {
                continue;
            };
            let invalid = if self.rule == "JoinSetBehindLock" {
                self.context
                    .resolve_external_type_paths(resolved.index, resolved.module, inner)
                    .iter()
                    .any(|identity| identity.format_path() == "tokio::task::JoinSet")
            } else {
                self.context.resolve_type_syntax(resolved.index, resolved.module, inner).iter().any(|inner|
                    matches!(&inner.typed, syn::Type::Path(path) if path.path.get_ident().is_some_and(|name|
                        ["bool", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize"].contains(&name.to_string().as_str()))))
            };
            if invalid {
                self.report(typed.span());
                break;
            }
        }
    }

    fn check_update(&mut self, expression: &syn::ExprMethodCall) {
        let atomic = expression.method == "fetch_update"
            && self.receiver_has(&expression.receiver, ATOMIC_TYPES);
        let watch = matches!(
            expression.method.to_string().as_str(),
            "send_modify" | "send_if_modified"
        ) && self.receiver_has(&expression.receiver, &["tokio::sync::watch::Sender"]);
        if !atomic && !watch {
            return;
        }
        for argument in &expression.args {
            if !matches!(argument, syn::Expr::Closure(_)) {
                continue;
            }
            let effects = ClosureEffects::collect_nested(argument);
            let impure = atomic && effects.assigns_capture || (watch || atomic) && (
                effects.methods.iter().any(|call| {
                    let method = call.method.to_string();
                    let captured = effects.captured_methods.iter().any(|captured| captured.span().start() == call.span().start());
                    let lock = matches!(method.as_str(), "lock" | "blocking_lock" | "read" | "write" | "blocking_read" | "blocking_write")
                        && self.receiver_has(&call.receiver, LOCK_TYPES);
                    let atomic_write = captured
                        && ATOMIC_WRITES.contains(&method.as_str())
                        && self.receiver_has(&call.receiver, ATOMIC_TYPES);
                    let mutation = atomic
                        && captured
                        && matches!(method.as_str(), "push" | "pop" | "insert" | "remove" | "clear" | "extend" | "truncate" | "retain" | "resize" | "append" | "drain" | "sort" | "reverse" | "swap_remove")
                        && self.receiver_has(&call.receiver, &["std::vec::Vec", "alloc::vec::Vec", "std::collections::HashMap", "std::collections::HashSet", "std::collections::BTreeMap", "std::collections::BTreeSet", "std::collections::VecDeque"]);
                    let channel = matches!(method.as_str(), "send" | "try_send" | "send_replace" | "send_modify" | "send_if_modified")
                        && self.receiver_has(&call.receiver, &["tokio::sync::watch::Sender", "tokio::sync::mpsc::Sender", "tokio::sync::mpsc::UnboundedSender", "tokio::sync::broadcast::Sender", "tokio::sync::oneshot::Sender", "std::sync::mpsc::Sender", "std::sync::mpsc::SyncSender"]);
                    lock || channel || mutation || atomic_write
                }) || effects.calls.iter().any(|call| matches!(&*call.func, syn::Expr::Path(path)
                    if self.context.resolve_external_call_paths(self.ordinal, self.module, &path.path).iter().any(|path|
                        ["std::fs::", "tokio::fs::", "std::net::", "tokio::net::"].iter().any(|prefix| path.format_path().starts_with(prefix))))));
            if impure {
                self.report(argument.span());
            }
        }
    }

    fn unsafe_select_call(&self, call: &syn::ExprMethodCall) -> bool {
        if call.method == "lock" {
            return self.receiver_has(&call.receiver, &["tokio::sync::Mutex"]);
        }
        if !matches!(
            call.method.to_string().as_str(),
            "read_exact" | "read_to_end" | "write_all"
        ) {
            return false;
        }
        self.receiver_types(&call.receiver)
            .iter()
            .flat_map(|receiver| {
                self.context.resolve_external_type_paths(
                    receiver.index,
                    receiver.module,
                    &receiver.typed,
                )
            })
            .any(|path| {
                path.package == "tokio"
                    && path
                        .segments
                        .iter()
                        .any(|segment| matches!(segment.as_str(), "io" | "fs" | "net"))
            })
    }

    fn check_guard_tail(
        &mut self,
        local: &syn::Local,
        call: &syn::ExprMethodCall,
        name: &str,
        following: &[syn::Stmt],
    ) {
        if self.rule == "CriticalSectionWrite" {
            let collections = self
                .receiver_types(&call.receiver)
                .iter()
                .flat_map(|receiver| {
                    crate::runtime::util::guard_collection_fields(
                        self.context,
                        receiver.index,
                        receiver.module,
                        &receiver.typed,
                        0,
                    )
                })
                .collect::<Vec<_>>();
            let writes = following
                .iter()
                .map(|statement| GuardWrites::count(statement, name, &collections))
                .collect::<Vec<_>>();
            let count = writes.iter().sum::<usize>();
            let tail_value = usize::from(
                matches!(
                    following.last(),
                    Some(syn::Stmt::Expr(syn::Expr::Path(_), None))
                ) || matches!(following.last(), Some(syn::Stmt::Expr(syn::Expr::Return(returned), _))
                    if returned.expr.as_ref().is_some_and(|value| matches!(&**value, syn::Expr::Path(_) | syn::Expr::Lit(_)))),
            );
            let trailing = writes
                .iter()
                .position(|count| *count > 0)
                .is_some_and(|write| write + 1 < writes.len() - tail_value);
            if count > 1 || trailing {
                self.report(local.span());
            }
        } else if self.receiver_has(&call.receiver, &["tokio::sync::Mutex"])
            && !following
                .iter()
                .take_while(|statement| !self.drops_guard(statement, name))
                .any(AwaitVisitor::in_statement)
        {
            self.report(local.span());
        }
    }

    fn guard_binding(pattern: &syn::Pat) -> Option<&syn::Ident> {
        match pattern {
            syn::Pat::Ident(name) => Some(&name.ident),
            syn::Pat::Type(typed) => Self::guard_binding(&typed.pat),
            _ => None,
        }
    }

    fn temporary_guard_without_await(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Await(awaited) => {
                matches!(&*awaited.base, syn::Expr::MethodCall(call)
                    if call.method == "lock"
                        && self.receiver_has(&call.receiver, &["tokio::sync::Mutex"]))
            }
            syn::Expr::MethodCall(call) => {
                !call.args.iter().any(AwaitVisitor::in_expression)
                    && self.temporary_guard_without_await(&call.receiver)
            }
            syn::Expr::Field(field) => self.temporary_guard_without_await(&field.base),
            syn::Expr::Paren(paren) => self.temporary_guard_without_await(&paren.expr),
            syn::Expr::Reference(reference) => self.temporary_guard_without_await(&reference.expr),
            syn::Expr::Unary(unary) => self.temporary_guard_without_await(&unary.expr),
            _ => false,
        }
    }

    fn is_channel_send(&self, expression: &syn::Expr) -> bool {
        let syn::Expr::MethodCall(call) = expression else {
            return false;
        };
        match call.method.to_string().as_str() {
            "send" => self.receiver_has(
                &call.receiver,
                &[
                    "tokio::sync::oneshot::Sender",
                    "tokio::sync::mpsc::UnboundedSender",
                    "tokio::sync::broadcast::Sender",
                ],
            ),
            "try_send" => self.receiver_has(&call.receiver, &["tokio::sync::mpsc::Sender"]),
            _ => false,
        }
    }

    fn drops_guard(&self, statement: &syn::Stmt, name: &str) -> bool {
        let syn::Stmt::Expr(syn::Expr::Call(call), _) = statement else {
            return false;
        };
        let syn::Expr::Path(path) = &*call.func else {
            return false;
        };
        self.context
            .resolve_external_call_paths(self.ordinal, self.module, &path.path)
            .iter()
            .any(|identity| {
                matches!(
                    identity.format_path().as_str(),
                    "core::mem::drop" | "std::mem::drop"
                )
            })
            && call.args.iter().any(
                |argument| matches!(argument, syn::Expr::Path(path) if path.path.is_ident(name)),
            )
    }

    fn returns_true(expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Lit(literal) => matches!(&literal.lit, syn::Lit::Bool(value) if value.value),
            syn::Expr::Block(block) => {
                matches!(block.block.stmts.last(), Some(syn::Stmt::Expr(expression, _)) if Self::returns_true(expression))
            }
            syn::Expr::Return(returned) => returned
                .expr
                .as_ref()
                .is_some_and(|expression| Self::returns_true(expression)),
            syn::Expr::Paren(paren) => Self::returns_true(&paren.expr),
            syn::Expr::Binary(binary) if matches!(binary.op, syn::BinOp::Or(_)) => {
                Self::returns_true(&binary.left) || Self::returns_true(&binary.right)
            }
            syn::Expr::Binary(binary) if matches!(binary.op, syn::BinOp::And(_)) => {
                Self::returns_true(&binary.left) && Self::returns_true(&binary.right)
            }
            syn::Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Not(_)) => {
                matches!(&*unary.expr, syn::Expr::Lit(literal) if matches!(&literal.lit, syn::Lit::Bool(value) if !value.value))
            }
            _ => false,
        }
    }

    fn duration_roundtrip(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Cast(cast) => self.duration_roundtrip(&cast.expr),
            syn::Expr::Paren(paren) => self.duration_roundtrip(&paren.expr),
            syn::Expr::Binary(binary)
                if matches!(
                    binary.op,
                    syn::BinOp::Add(_)
                        | syn::BinOp::Sub(_)
                        | syn::BinOp::Mul(_)
                        | syn::BinOp::Div(_)
                ) =>
            {
                self.duration_roundtrip(&binary.left) || self.duration_roundtrip(&binary.right)
            }
            syn::Expr::MethodCall(call) => {
                call.method.to_string().starts_with("as_")
                    && self.receiver_has(
                        &call.receiver,
                        &[
                            "std::time::Duration",
                            "core::time::Duration",
                            "tokio::time::Duration",
                        ],
                    )
            }
            _ => false,
        }
    }

    fn invalid_timeout(&self, expression: &syn::ExprCall) -> bool {
        let syn::Expr::Path(path) = &*expression.func else {
            return false;
        };
        if !self
            .context
            .resolve_external_call_paths(self.ordinal, self.module, &path.path)
            .iter()
            .any(|path| path.format_path() == "tokio::time::timeout")
        {
            return false;
        }
        let Some(future) = expression.args.iter().nth(1) else {
            return false;
        };
        if matches!(future, syn::Expr::Async(_)) {
            return !AwaitVisitor::in_expression(future);
        }
        if let syn::Expr::Call(call) = future
            && let syn::Expr::Path(path) = &*call.func
            && self
                .context
                .resolve_external_call_paths(self.ordinal, self.module, &path.path)
                .iter()
                .any(|path| {
                    matches!(
                        path.format_path().as_str(),
                        "std::future::ready" | "core::future::ready"
                    )
                })
        {
            return true;
        }
        let syn::Expr::MethodCall(call) = future else {
            return false;
        };
        if call.method != "wait_for"
            || !self.receiver_has(&call.receiver, &["tokio::sync::watch::Receiver"])
        {
            return false;
        }
        call.args.iter().any(|argument| matches!(argument, syn::Expr::Closure(closure) if Self::returns_true(&closure.body)))
    }

    fn lag_pattern(&self, pattern: &syn::Pat) -> bool {
        match pattern {
            syn::Pat::Wild(_) | syn::Pat::Ident(_) => true,
            syn::Pat::Or(pattern) => pattern.cases.iter().any(|case| self.lag_pattern(case)),
            syn::Pat::TupleStruct(pattern) => {
                if crate::runtime::util::is_result_variant(
                    self.context,
                    self.ordinal,
                    self.module,
                    &pattern.path,
                    "Err",
                ) {
                    return pattern
                        .elems
                        .iter()
                        .any(|pattern| self.lag_pattern(pattern));
                }
                self.context
                    .resolve_external_call_paths(self.ordinal, self.module, &pattern.path)
                    .iter()
                    .any(|path| {
                        path.format_path() == "tokio::sync::broadcast::error::RecvError::Lagged"
                    })
            }
            _ => false,
        }
    }

    fn broadcast_error(&self, expression: &syn::Expr) -> bool {
        self.receiver_types(expression).iter().any(|receiver| {
            let syn::Type::Path(path) = &receiver.typed else {
                return false;
            };
            let Some(segment) = path.path.segments.last() else {
                return false;
            };
            let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                return false;
            };
            let Some(syn::GenericArgument::Type(error)) = arguments.args.iter().nth(1) else {
                return false;
            };
            self.receiver_identity(
                receiver.index,
                receiver.module,
                error,
                &crate::source::model::CfgPredicate::All(Vec::new()),
                &["tokio::sync::broadcast::error::RecvError"],
                0,
            )
        })
    }

    fn broadcast_recv(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Paren(paren) => self.broadcast_recv(&paren.expr),
            syn::Expr::Group(group) => self.broadcast_recv(&group.expr),
            syn::Expr::MethodCall(call) if call.method == "ok" => {
                self.broadcast_recv(&call.receiver)
            }
            syn::Expr::Await(awaited) => !self.awaited_types(awaited).is_empty(),
            syn::Expr::Path(_) => self.broadcast_error(expression),
            _ => false,
        }
    }

    fn lag_stopping_let(&self, pattern: &syn::Pat, expression: &syn::Expr) -> bool {
        let syn::Pat::TupleStruct(pattern) = pattern else {
            return false;
        };
        let value = pattern.path.is_ident("Some")
            || crate::runtime::util::is_result_variant(
                self.context,
                self.ordinal,
                self.module,
                &pattern.path,
                "Ok",
            );
        value && self.broadcast_recv(expression)
    }

    fn let_condition(condition: &syn::Expr) -> Option<&syn::ExprLet> {
        match condition {
            syn::Expr::Let(expression) => Some(expression),
            syn::Expr::Paren(paren) => Self::let_condition(&paren.expr),
            syn::Expr::Binary(binary) if matches!(binary.op, syn::BinOp::And(_)) => {
                Self::let_condition(&binary.left).or_else(|| Self::let_condition(&binary.right))
            }
            _ => None,
        }
    }

    fn runs_off_async_call(&self, call: &syn::ExprCall) -> bool {
        let syn::Expr::Path(path) = &*call.func else {
            return false;
        };
        self.context
            .resolve_external_call_paths(self.ordinal, self.module, &path.path)
            .iter()
            .any(|path| TaskVisitor::runs_off_async(&path.format_path()))
    }

    fn visit_closure_argument(&mut self, argument: &syn::Expr, inherit: bool) {
        self.inherit_async = inherit && matches!(argument, syn::Expr::Closure(_));
        self.visit_expr(argument);
        self.inherit_async = false;
    }

    fn terminates_loop(expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Break(_) | syn::Expr::Return(_) => true,
            syn::Expr::Match(branch) => branch
                .arms
                .iter()
                .any(|arm| Self::terminates_loop(&arm.body)),
            syn::Expr::Block(block) => block.block.stmts.iter().any(|statement| matches!(statement, syn::Stmt::Expr(expression, _) if Self::terminates_loop(expression))),
            syn::Expr::If(branch) => branch.then_branch.stmts.iter().any(|statement| matches!(statement, syn::Stmt::Expr(expression, _) if Self::terminates_loop(expression))) || branch.else_branch.as_ref().is_some_and(|(_, expression)| Self::terminates_loop(expression)),
            _ => false,
        }
    }
}

impl<'ast> Visit<'ast> for TypedRuntimeVisitor<'_> {
    fn visit_expr_match(&mut self, expression: &'ast syn::ExprMatch) {
        if self.rule == "BroadcastLag" && self.loops > 0 && self.broadcast_recv(&expression.expr) {
            for arm in &expression.arms {
                if self.lag_pattern(&arm.pat) && Self::terminates_loop(&arm.body) {
                    self.report(arm.span());
                }
            }
        }
        self.visit_expr(&expression.expr);
        let success = self.result_types(&expression.expr);
        for arm in &expression.arms {
            let length = self.bindings.len();
            self.bind_pattern(&arm.pat);
            if let syn::Pat::TupleStruct(pattern) = &arm.pat
                && crate::runtime::util::is_result_variant(
                    self.context,
                    self.ordinal,
                    self.module,
                    &pattern.path,
                    "Ok",
                )
                && let Some(syn::Pat::Ident(binding)) = pattern.elems.first()
                && let Some(typed) = success.first()
            {
                self.bindings
                    .push((binding.ident.to_string(), Rc::clone(typed)));
            }
            syn::visit::visit_arm(self, arm);
            self.bindings.truncate(length);
        }
    }
    fn visit_expr_closure(&mut self, expression: &'ast syn::ExprClosure) {
        let asynchronous = self.async_context;
        let inherited = std::mem::take(&mut self.inherit_async);
        self.async_context = expression.asyncness.is_some() || inherited;
        let length = self.bindings.len();
        let parent = (self.synchronous_boundary, self.loops);
        self.synchronous_boundary = false;
        self.loops = 0;
        for pattern in &expression.inputs {
            self.bind_pattern(pattern);
        }
        syn::visit::visit_expr_closure(self, expression);
        self.bindings.truncate(length);
        (self.synchronous_boundary, self.loops) = parent;
        self.async_context = asynchronous;
    }

    fn visit_expr_async(&mut self, expression: &'ast syn::ExprAsync) {
        let asynchronous = self.async_context;
        self.async_context = true;
        let parent = (self.synchronous_boundary, self.loops);
        self.synchronous_boundary = false;
        self.loops = 0;
        syn::visit::visit_expr_async(self, expression);
        (self.synchronous_boundary, self.loops) = parent;
        self.async_context = asynchronous;
    }
    fn visit_expr_loop(&mut self, expression: &'ast syn::ExprLoop) {
        self.loops += 1;
        syn::visit::visit_expr_loop(self, expression);
        self.loops -= 1;
    }

    fn visit_expr_for_loop(&mut self, expression: &'ast syn::ExprForLoop) {
        self.loops += 1;
        syn::visit::visit_expr_for_loop(self, expression);
        self.loops -= 1;
    }

    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        if self.rule != "SelectCancelSafety" || self.loops == 0 {
            return;
        }
        if !self
            .context
            .resolve_external_call_paths(self.ordinal, self.module, &item.path)
            .iter()
            .any(|path| path.format_path() == "tokio::select")
        {
            return;
        }
        let Ok(branches) = syn::parse2::<SelectBranches>(item.tokens.clone()) else {
            return;
        };
        for future in branches.futures {
            let effects = ClosureEffects::collect(&future);
            let qualified = effects.calls.iter().any(|call| matches!(&*call.func, syn::Expr::Path(path)
                if self.context.resolve_external_call_paths(self.ordinal, self.module, &path.path).iter().any(|path|
                    matches!(path.format_path().as_str(), "tokio::sync::Mutex::lock" | "tokio::io::AsyncReadExt::read_exact" | "tokio::io::AsyncReadExt::read_to_end" | "tokio::io::AsyncWriteExt::write_all"))));
            if qualified
                || effects
                    .methods
                    .iter()
                    .any(|call| self.unsafe_select_call(call))
            {
                self.report(future.span());
            }
        }
    }
    fn visit_type(&mut self, typed: &'ast syn::Type) {
        if matches!(self.rule, "JoinSetBehindLock" | "LockChoice") {
            self.check_lock_type(typed);
        }
        syn::visit::visit_type(self, typed);
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

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let expectation = self.thread_expect;
        self.thread_expect |= expects_lint(&item.attrs, "disallowed_methods");
        let parent = self.self_type.replace((*item.self_ty).clone());
        syn::visit::visit_item_impl(self, item);
        self.self_type = parent;
        self.thread_expect = expectation;
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        let runtime_main = self.runtime_main;
        self.runtime_main = item.sig.ident == "main"
            && self.context.indexes[self.ordinal].modules[self.module]
                .module
                .path
                .is_empty()
            && self.context.project_config.crates.iter().any(|config| {
                config.name == self.context.indexes[self.ordinal].crate_name
                    && config.role == Some(crate::config::model::Role::EntryPoint)
            });
        let asynchronous = self.async_context;
        self.async_context = item.sig.asyncness.is_some();
        let expectation = self.thread_expect;
        self.thread_expect |= expects_lint(&item.attrs, "disallowed_methods");
        let parent = std::mem::take(&mut self.bindings);
        let boundary = self.synchronous_boundary;
        self.synchronous_boundary = item.sig.asyncness.is_none() && item.sig.ident == "on_disable";
        self.bind_signature(&item.sig);
        syn::visit::visit_item_fn(self, item);
        self.bindings = parent;
        self.synchronous_boundary = boundary;
        self.thread_expect = expectation;
        self.async_context = asynchronous;
        self.runtime_main = runtime_main;
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        let asynchronous = self.async_context;
        self.async_context = item.sig.asyncness.is_some();
        let expectation = self.thread_expect;
        self.thread_expect |= expects_lint(&item.attrs, "disallowed_methods");
        let parent = std::mem::take(&mut self.bindings);
        let boundary = self.synchronous_boundary;
        self.synchronous_boundary = item.sig.asyncness.is_none()
            && (item.sig.ident == "drop" || item.sig.ident == "on_disable");
        self.bind_signature(&item.sig);
        syn::visit::visit_impl_item_fn(self, item);
        self.bindings = parent;
        self.synchronous_boundary = boundary;
        self.thread_expect = expectation;
        self.async_context = asynchronous;
    }

    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        let bindings = std::mem::take(&mut self.bindings);
        let parent = (self.synchronous_boundary, self.async_context);
        self.synchronous_boundary = item.sig.asyncness.is_none() && item.sig.ident == "on_disable";
        self.async_context = item.sig.asyncness.is_some();
        self.bind_signature(&item.sig);
        syn::visit::visit_trait_item_fn(self, item);
        self.bindings = bindings;
        (self.synchronous_boundary, self.async_context) = parent;
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        let length = self.bindings.len();
        for (position, statement) in block.stmts.iter().enumerate() {
            self.visit_stmt(statement);
            if !matches!(self.rule, "CriticalSectionWrite" | "LockChoice") {
                continue;
            }
            if self.rule == "LockChoice"
                && let syn::Stmt::Expr(expression, Some(_)) = statement
                && self.temporary_guard_without_await(expression)
            {
                self.report(statement.span());
            }
            let syn::Stmt::Local(local) = statement else {
                continue;
            };
            let Some(name) = Self::guard_binding(&local.pat) else {
                continue;
            };
            let Some(init) = &local.init else {
                continue;
            };
            let base = match &*init.expr {
                syn::Expr::Await(awaited) => &*awaited.base,
                expression => expression,
            };
            let syn::Expr::MethodCall(call) = base else {
                continue;
            };
            let call = if matches!(
                call.method.to_string().as_str(),
                "unwrap" | "unwrap_or_else" | "expect"
            ) && let syn::Expr::MethodCall(lock) = &*call.receiver
                && matches!(lock.method.to_string().as_str(), "lock" | "write")
                && self.receiver_has(&lock.receiver, &["std::sync::Mutex", "std::sync::RwLock"])
            {
                lock
            } else {
                call
            };
            if !matches!(call.method.to_string().as_str(), "lock" | "write")
                || !self.receiver_has(
                    &call.receiver,
                    &[
                        "tokio::sync::Mutex",
                        "tokio::sync::RwLock",
                        "parking_lot::Mutex",
                        "parking_lot::RwLock",
                        "std::sync::Mutex",
                        "std::sync::RwLock",
                    ],
                )
            {
                continue;
            }
            let following = &block.stmts[position + 1..];
            self.check_guard_tail(local, call, &name.to_string(), following);
        }
        self.bindings.truncate(length);
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        let expectation = self.thread_expect;
        self.thread_expect |= expects_lint(&local.attrs, "disallowed_methods");
        if self.rule == "ResultConsumption"
            && expects_lint(&local.attrs, "let_underscore_must_use")
            && matches!(local.pat, syn::Pat::Wild(_))
            && let Some(init) = &local.init
            && !self.is_channel_send(&init.expr)
        {
            self.report(local.span());
        }
        if self.rule == "ResultConsumption"
            && let syn::Pat::Ident(name) = &local.pat
            && name.ident.to_string().starts_with('_')
            && let Some(init) = &local.init
            && self.is_channel_send(&init.expr)
        {
            self.report(local.span());
        }
        if self.rule == "BroadcastLag"
            && self.loops > 0
            && let Some(init) = &local.init
            && let Some((_, diverge)) = &init.diverge
            && self.lag_stopping_let(&local.pat, &init.expr)
            && Self::terminates_loop(diverge)
        {
            self.report(local.span());
        }
        syn::visit::visit_local(self, local);
        self.bind_initializer(local);
        self.thread_expect = expectation;
    }

    fn visit_expr_method_call(&mut self, expression: &'ast syn::ExprMethodCall) {
        let expectation = self.thread_expect;
        self.thread_expect |= expects_lint(&expression.attrs, "disallowed_methods");
        if expression.method == "block_on"
            && self.receiver_has(
                &expression.receiver,
                &["tokio::runtime::Runtime", "tokio::runtime::Handle"],
            )
        {
            let index = &self.context.indexes[self.ordinal];
            let in_test = index.modules[self.module].tree != crate::source::model::SourceTree::Src;
            let outside_boundary = !self.runtime_main
                && !crate::declaration::util::is_technology(self.context, index, self.module);
            if self.rule == "BlockOnInTest" && in_test
                || self.rule == "BlockOnOutsideBoundary" && outside_boundary
            {
                self.report(expression.span());
            }
        }
        let blocking_async = self.rule == "BlockingCallInAsync" && self.async_context;
        if blocking_async && self.is_blocking_method(expression) {
            self.report(expression.span());
        }
        if self.rule == "ThreadPrimitive"
            && self.thread_expect
            && expression.method == "spawn"
            && self.receiver_has(&expression.receiver, &["std::thread::Builder"])
            && !crate::declaration::util::is_technology(
                self.context,
                &self.context.indexes[self.ordinal],
                self.module,
            )
        {
            self.report(expression.span());
        }
        if self.rule == "ImpureUpdateClosure" {
            self.check_update(expression);
        }
        if self.rule == "BlockingLockInSync"
            && self.synchronous_boundary
            && expression.method == "blocking_lock"
            && self.receiver_has(&expression.receiver, &["tokio::sync::Mutex"])
        {
            self.report(expression.span());
        }
        self.visit_expr(&expression.receiver);
        if let Some(turbofish) = &expression.turbofish {
            self.visit_angle_bracketed_generic_arguments(turbofish);
        }
        let inherit = self.async_context
            && !matches!(
                expression.method.to_string().as_str(),
                "spawn" | "spawn_blocking" | "spawn_scoped"
            );
        for argument in &expression.args {
            self.visit_closure_argument(argument, inherit);
        }
        self.thread_expect = expectation;
    }

    fn visit_expr_while(&mut self, expression: &'ast syn::ExprWhile) {
        self.loops += 1;
        if self.rule == "BroadcastLag"
            && let Some(condition) = Self::let_condition(&expression.cond)
            && self.lag_stopping_let(&condition.pat, &condition.expr)
        {
            self.report(expression.span());
        }
        let length = self.bindings.len();
        syn::visit::visit_expr_while(self, expression);
        self.bindings.truncate(length);
        self.loops -= 1;
    }

    fn visit_expr_if(&mut self, expression: &'ast syn::ExprIf) {
        let length = self.bindings.len();
        self.visit_expr(&expression.cond);
        self.visit_block(&expression.then_branch);
        self.bindings.truncate(length);
        if let Some((_, otherwise)) = &expression.else_branch {
            self.visit_expr(otherwise);
        }
    }

    fn visit_expr_let(&mut self, expression: &'ast syn::ExprLet) {
        self.visit_expr(&expression.expr);
        if !self.bind_payload(&expression.pat, &expression.expr) {
            self.bind_pattern(&expression.pat);
        }
    }

    fn visit_expr_call(&mut self, expression: &'ast syn::ExprCall) {
        if self.rule == "WatchTimeout" && self.invalid_timeout(expression) {
            self.report(expression.span());
        }
        if self.rule == "DurationOverloadCall"
            && let syn::Expr::Path(path) = &*expression.func
            && self
                .context
                .resolve_external_call_paths(self.ordinal, self.module, &path.path)
                .iter()
                .any(|resolved| {
                    [
                        "std::time::Duration::from_",
                        "core::time::Duration::from_",
                        "tokio::time::Duration::from_",
                    ]
                    .iter()
                    .any(|prefix| resolved.format_path().starts_with(prefix))
                })
            && expression
                .args
                .iter()
                .any(|argument| self.duration_roundtrip(argument))
        {
            self.report(expression.span());
        }
        self.visit_expr(&expression.func);
        let inherit = self.async_context && !self.runs_off_async_call(expression);
        for argument in &expression.args {
            self.visit_closure_argument(argument, inherit);
        }
    }
}
