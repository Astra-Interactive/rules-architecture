use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::{build_node_finding, format_module_path};
use crate::runtime::rule::AwaitVisitor;
use crate::runtime::rule::ClosureEffects;
use crate::runtime::rule::GuardWrites;
use crate::runtime::rule::RuntimeVisitor;
use crate::runtime::rule::SelectBranches;
use crate::runtime::rule::TaskVisitor;
use crate::runtime::util::expects_lint;
use crate::runtime::util::names_format_argument;
use crate::runtime::util::node_predicate;
use crate::source::model::ResolvedTypeSyntax;
use crate::source::model::SourceFile;
use crate::source::util::find_node_span;
use quote::ToTokens;
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
    in_drop: bool,
    loops: usize,
    thread_expect: bool,
    async_context: bool,
    runtime_main: bool,
    inherit_async: bool,
    closure_state: Option<Rc<ResolvedTypeSyntax>>,
    generic_scopes: Vec<syn::Generics>,
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
                    in_drop: false,
                    loops: 0,
                    thread_expect: expects_lint(&file.syntax.attrs, "disallowed_methods"),
                    async_context: false,
                    runtime_main: false,
                    inherit_async: false,
                    closure_state: None,
                    generic_scopes: Vec::new(),
                };
                visitor.visit_file(&file.syntax);
                findings.extend(visitor.findings);
            }
        }
        findings
    }

    fn report(&mut self, span: proc_macro2::Span, message: &str) {
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
            message.to_owned(),
        ));
    }

    fn report_discarded_result(&mut self, span: proc_macro2::Span) {
        self.report(span, Self::DISCARDED_RESULT_MESSAGE);
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
        if matches!(call.method.to_string().as_str(), "name" | "stack_size")
            && self.receiver_has(&call.receiver, &["std::thread::Builder"])
        {
            return self.receiver_types(&call.receiver);
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

    fn find_struct_field_types(
        &self,
        module: usize,
        typed: &syn::Type,
        member: &syn::Member,
    ) -> Vec<Rc<ResolvedTypeSyntax>> {
        self.context.indexes[self.ordinal]
            .resolve_type_items(module, typed)
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
                        let matching = match member {
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

    fn resolve_field_receiver_types(&self, field: &syn::ExprField) -> Vec<Rc<ResolvedTypeSyntax>> {
        if matches!(&*field.base, syn::Expr::Path(path) if path.path.is_ident("self")) {
            let Some(typed) = &self.self_type else {
                return Vec::new();
            };
            return self.find_struct_field_types(self.module, typed, &field.member);
        }
        self.receiver_types(&field.base)
            .iter()
            .flat_map(|base| self.find_struct_field_types(base.module, &base.typed, &field.member))
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

    fn build_owner_type(function: &syn::Path) -> syn::Type {
        let mut owner = function.clone();
        owner.segments.pop();
        owner.segments.pop_punct();
        syn::Type::Path(syn::TypePath {
            attrs: Vec::new(),
            qself: None,
            path: owner,
        })
    }

    fn resolve_awaited_connection_types(
        &self,
        awaited: &syn::ExprAwait,
    ) -> Vec<Rc<ResolvedTypeSyntax>> {
        let syn::Expr::Call(call) = &*awaited.base else {
            return Vec::new();
        };
        let syn::Expr::Path(path) = &*call.func else {
            return Vec::new();
        };
        let connects = self
            .context
            .resolve_external_call_paths(self.ordinal, self.module, &path.path)
            .iter()
            .any(|path| {
                matches!(
                    path.format_path().as_str(),
                    "tokio::net::TcpStream::connect"
                        | "tokio::net::UnixStream::connect"
                        | "tokio::fs::File::open"
                        | "tokio::fs::File::create"
                )
            });
        if !connects {
            return Vec::new();
        }
        let typed = Self::build_owner_type(&path.path);
        vec![self.build_receiver_type(
            syn::parse_quote!(::std::result::Result<#typed, _>),
            self.module,
        )]
    }

    fn resolve_awaited_receiver_types(
        &self,
        awaited: &syn::ExprAwait,
    ) -> Vec<Rc<ResolvedTypeSyntax>> {
        let received = self.awaited_types(awaited);
        if received.is_empty() {
            return self.resolve_awaited_connection_types(awaited);
        }
        received
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
            syn::Expr::Await(awaited) => self.resolve_awaited_receiver_types(awaited),
            syn::Expr::Path(path) => self
                .bindings
                .iter()
                .rev()
                .find(|(name, _)| path.path.is_ident(name))
                .map(|(_, typed)| vec![Rc::clone(typed)])
                .unwrap_or_default(),
            syn::Expr::Field(field) => self.resolve_field_receiver_types(field),
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

    fn is_block_on_receiver(&self, expression: &syn::Expr) -> bool {
        self.receiver_has(
            expression,
            &["tokio::runtime::Runtime", "tokio::runtime::Handle"],
        ) || matches!(self.rule, "BlockOnOutsideBoundary" | "BlockOnInTest")
            && self.receiver_has(expression, &["tokio::task::LocalSet"])
    }

    fn is_local_pool_run(&self, expression: &syn::ExprMethodCall) -> bool {
        let local_pools = [
            "futures::executor::LocalPool",
            "futures_executor::LocalPool",
            "futures_executor::local_pool::LocalPool",
        ];
        expression.method == "run_until" && self.receiver_has(&expression.receiver, &local_pools)
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

    fn is_blocking_lock_call(&self, call: &syn::ExprMethodCall) -> bool {
        let owner = match call.method.to_string().as_str() {
            "blocking_lock" => "tokio::sync::Mutex",
            "blocking_read" | "blocking_write" => "tokio::sync::RwLock",
            _ => return false,
        };
        self.receiver_has(&call.receiver, &[owner])
    }

    fn is_blocking_channel_call(&self, call: &syn::ExprMethodCall) -> bool {
        let owners: &[&str] = match call.method.to_string().as_str() {
            "recv" | "recv_timeout" => &["std::sync::mpsc::Receiver"],
            "blocking_recv" => &[
                "tokio::sync::mpsc::Receiver",
                "tokio::sync::mpsc::UnboundedReceiver",
                "tokio::sync::broadcast::Receiver",
            ],
            "blocking_send" => &["tokio::sync::mpsc::Sender"],
            _ => return false,
        };
        self.receiver_has(&call.receiver, owners)
    }

    fn check_blocking_in_sync(&mut self, call: &syn::ExprMethodCall) {
        let is_lock = self.synchronous_boundary && self.is_blocking_lock_call(call);
        let is_channel = self.in_drop && self.is_blocking_channel_call(call);
        if is_lock {
            self.report(call.span(), Self::BLOCKING_LOCK_MESSAGE);
        }
        if is_channel {
            self.report(call.span(), Self::BLOCKING_CHANNEL_MESSAGE);
        }
    }

    fn is_blocking_io_method(&self, call: &syn::ExprMethodCall) -> bool {
        let method = call.method.to_string();
        let method = method.as_str();
        let file_call = Self::FILE_BLOCKING_METHODS.contains(&method)
            && self.receiver_has(&call.receiver, &["std::fs::File"]);
        let stream_call = Self::TCP_STREAM_IO_METHODS.contains(&method)
            && self.receiver_has(&call.receiver, &["std::net::TcpStream"]);
        let command_call = Self::COMMAND_IO_METHODS.contains(&method)
            && self.receiver_has(&call.receiver, &["std::process::Command"]);
        file_call || stream_call || command_call
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
        blocking_channel || self.is_blocking_io_method(expression)
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
        let typed = Self::build_owner_type(&path.path);
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
            let is_join_set_rule = self.rule == "JoinSetBehindLock";
            let invalid = if is_join_set_rule {
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
                let message = if is_join_set_rule {
                    Self::JOIN_SET_BEHIND_LOCK_MESSAGE
                } else {
                    Self::LOCK_AROUND_PRIMITIVE_MESSAGE
                };
                self.report(typed.span(), message);
                break;
            }
        }
    }

    fn is_io_call_path(path: &str) -> bool {
        let file_call = ["std::fs::", "tokio::fs::"]
            .iter()
            .find_map(|prefix| path.strip_prefix(prefix))
            .is_some_and(|rest| {
                !rest.contains("::")
                    || ["File::open", "File::create", "File::create_new"].contains(&rest)
            });
        let network_call = ["std::net::", "tokio::net::"]
            .iter()
            .find_map(|prefix| path.strip_prefix(prefix))
            .is_some_and(|rest| {
                rest == "lookup_host"
                    || [
                        "TcpStream::",
                        "TcpListener::",
                        "UdpSocket::",
                        "UnixStream::",
                        "UnixListener::",
                        "UnixDatagram::",
                    ]
                    .iter()
                    .any(|socket| rest.starts_with(socket))
            });
        file_call || network_call
    }

    fn resolves_to_any(&self, path: &syn::Path, names: &[&str]) -> bool {
        self.context
            .resolve_external_call_paths(self.ordinal, self.module, path)
            .iter()
            .any(|resolved| names.contains(&resolved.format_path().as_str()))
    }

    fn has_sleep_or_log_event(&self, effects: &ClosureEffects) -> bool {
        let sleeps = effects.calls.iter().any(|call| {
            matches!(&*call.func, syn::Expr::Path(path)
                if self.resolves_to_any(&path.path, &["std::thread::sleep"]))
        });
        let logs = effects
            .macros
            .iter()
            .any(|item| self.resolves_to_any(&item.path, Self::LOG_EVENT_MACROS));
        sleeps || logs
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
            let impure = atomic && (effects.assigns_capture || effects.borrows_capture_mutably)
                || (watch || atomic) && (
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
                    let option_write = atomic
                        && captured
                        && matches!(method.as_str(), "take" | "replace" | "insert" | "get_or_insert" | "get_or_insert_with")
                        && self.receiver_has(&call.receiver, OPTION_TYPES);
                    let channel = self.receiver_has(&call.receiver, Self::CHANNEL_ENDPOINT_TYPES);
                    lock || channel || mutation || option_write || atomic_write || self.is_blocking_io_method(call)
                }) || effects.calls.iter().any(|call| matches!(&*call.func, syn::Expr::Path(path)
                    if self.context.resolve_external_call_paths(self.ordinal, self.module, &path.path).iter().any(|path|
                        Self::is_io_call_path(&path.format_path())
                            || Self::STD_IO_FUNCTIONS.contains(&path.format_path().as_str())))));
            let is_noisy = watch && self.has_sleep_or_log_event(&effects);
            if impure || is_noisy {
                self.report(argument.span(), Self::IMPURE_UPDATE_MESSAGE);
            }
        }
    }

    fn is_cancel_unsafe_io_method(method: &str) -> bool {
        let fixed = [
            "read_exact",
            "read_to_end",
            "read_to_string",
            "write_all",
            "read_line",
            "read_until",
            "write_all_buf",
            "write_vectored",
            "next_line",
            "next_segment",
        ]
        .contains(&method);
        let numeric = ["read_", "write_"]
            .iter()
            .find_map(|prefix| method.strip_prefix(prefix))
            .map(|rest| rest.strip_suffix("_le").unwrap_or(rest))
            .is_some_and(|number| {
                [
                    "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32",
                    "f64",
                ]
                .contains(&number)
            });
        fixed || numeric
    }

    fn list_parameter_bounds(&self, name: &syn::Ident) -> Vec<&syn::TypeParamBound> {
        let mut bounds = Vec::new();
        for generics in &self.generic_scopes {
            for parameter in generics
                .type_params()
                .filter(|parameter| parameter.ident == *name)
            {
                bounds.extend(parameter.bounds.iter());
            }
            let predicates = generics
                .where_clause
                .iter()
                .flat_map(|clause| &clause.predicates);
            for predicate in predicates {
                let syn::WherePredicate::Type(predicate) = predicate else {
                    continue;
                };
                if matches!(&predicate.bounded_ty, syn::Type::Path(path) if path.path.is_ident(name))
                {
                    bounds.extend(predicate.bounds.iter());
                }
            }
        }
        bounds
    }

    fn is_tokio_io_bound(&self, bound: &syn::TypeParamBound) -> bool {
        let syn::TypeParamBound::Trait(bound) = bound else {
            return false;
        };
        let resolved =
            match self.context.indexes[self.ordinal].resolve_path(self.module, &bound.path) {
                crate::source::model::Resolution::External(path) => path,
                _ => return false,
            };
        [
            "tokio::io::AsyncRead",
            "tokio::io::AsyncWrite",
            "tokio::io::AsyncBufRead",
            "tokio::io::AsyncReadExt",
            "tokio::io::AsyncWriteExt",
            "tokio::io::AsyncBufReadExt",
        ]
        .contains(&resolved.as_str())
    }

    fn is_tokio_io_bounded(&self, typed: &syn::Type) -> bool {
        match typed {
            syn::Type::Reference(reference) => self.is_tokio_io_bounded(&reference.elem),
            syn::Type::Paren(paren) => self.is_tokio_io_bounded(&paren.elem),
            syn::Type::ImplTrait(implemented) => implemented
                .bounds
                .iter()
                .any(|bound| self.is_tokio_io_bound(bound)),
            syn::Type::Path(path) => path.path.get_ident().is_some_and(|name| {
                self.list_parameter_bounds(name)
                    .into_iter()
                    .any(|bound| self.is_tokio_io_bound(bound))
            }),
            _ => false,
        }
    }

    fn unsafe_select_call(&self, call: &syn::ExprMethodCall) -> bool {
        let queued = match call.method.to_string().as_str() {
            "lock" => Some(&["tokio::sync::Mutex"][..]),
            "read" | "write" => Some(&["tokio::sync::RwLock"][..]),
            "acquire" => Some(&["tokio::sync::Semaphore"][..]),
            "notified" => Some(&["tokio::sync::Notify"][..]),
            _ => None,
        };
        if let Some(names) = queued {
            return self.receiver_has(&call.receiver, names);
        }
        if !Self::is_cancel_unsafe_io_method(&call.method.to_string()) {
            return false;
        }
        let receivers = self.receiver_types(&call.receiver);
        let is_tokio_io = receivers
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
            });
        is_tokio_io
            || receivers
                .iter()
                .any(|receiver| self.is_tokio_io_bounded(&receiver.typed))
    }

    fn awaits_unsafe_call(
        &self,
        ordinal: usize,
        module: usize,
        signature: &syn::Signature,
        block: &syn::Block,
    ) -> bool {
        if signature.asyncness.is_none() {
            return false;
        }
        let mut callee = TypedRuntimeVisitor {
            context: self.context,
            ordinal,
            module,
            file: self.file,
            rule: self.rule,
            findings: Vec::new(),
            bindings: Vec::new(),
            self_type: self.self_type.clone(),
            synchronous_boundary: false,
            in_drop: false,
            loops: 0,
            thread_expect: false,
            async_context: true,
            runtime_main: false,
            inherit_async: false,
            closure_state: None,
            generic_scopes: vec![signature.generics.clone()],
        };
        callee.bind_signature(signature);
        let body = syn::Expr::Block(syn::ExprBlock {
            attrs: Vec::new(),
            label: None,
            block: block.clone(),
        });
        ClosureEffects::collect(&body)
            .methods
            .iter()
            .any(|call| callee.unsafe_select_call(call))
    }

    fn has_async_lock_type(&self, receiver: &syn::Expr) -> bool {
        self.receiver_has(receiver, &["tokio::sync::Mutex", "tokio::sync::RwLock"])
    }

    fn names_value(expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Lit(_) | syn::Expr::Path(_) => true,
            syn::Expr::Paren(paren) => Self::names_value(&paren.expr),
            syn::Expr::Unary(unary) => {
                matches!(unary.op, syn::UnOp::Neg(_)) && Self::names_value(&unary.expr)
            }
            syn::Expr::Field(field) => Self::names_value(&field.base),
            _ => false,
        }
    }

    fn has_tail_value(statements: &[syn::Stmt]) -> bool {
        match statements.last() {
            Some(syn::Stmt::Expr(syn::Expr::Return(returned), _)) => returned
                .expr
                .as_ref()
                .is_some_and(|value| Self::names_value(value)),
            Some(syn::Stmt::Expr(value, None)) => Self::names_value(value),
            _ => false,
        }
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
            let aliases = GuardWrites::collect_aliases(following, name);
            let writes = following
                .iter()
                .map(|statement| GuardWrites::count(statement, name, &collections, &aliases))
                .collect::<Vec<_>>();
            let count = writes.iter().sum::<usize>();
            let tail_value = usize::from(Self::has_tail_value(following));
            let trailing = writes
                .iter()
                .position(|count| *count > 0)
                .is_some_and(|write| write + 1 < writes.len() - tail_value);
            if count > 1 || trailing {
                self.report(local.span(), Self::CRITICAL_SECTION_WRITE_MESSAGE);
            }
        } else if self.has_async_lock_type(&call.receiver)
            && !following
                .iter()
                .take_while(|statement| !self.drops_guard(statement, name))
                .any(AwaitVisitor::in_statement)
        {
            self.report(local.span(), Self::ASYNC_LOCK_WITHOUT_AWAIT_MESSAGE);
        }
    }

    fn is_guard_method(&self, call: &syn::ExprMethodCall) -> bool {
        matches!(call.method.to_string().as_str(), "lock" | "write")
            || self.rule == "LockChoice" && call.method == "read"
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

    fn returns_workspace_result(
        &self,
        ordinal: usize,
        module: usize,
        signature: &syn::Signature,
        is_awaited: bool,
    ) -> bool {
        let syn::ReturnType::Type(_, returned) = &signature.output else {
            return false;
        };
        let returned_paths = self
            .context
            .resolve_outer_external_type_paths(ordinal, module, returned);
        let returns_result = signature.asyncness.is_some() == is_awaited
            && returned_paths
                .iter()
                .any(|path| path.segments.last().is_some_and(|name| name == "Result"));
        let returns_join_handle = is_awaited
            && signature.asyncness.is_none()
            && returned_paths
                .iter()
                .any(|path| path.format_path() == "tokio::task::JoinHandle");
        returns_result || returns_join_handle
    }

    fn has_result_method(
        &self,
        ordinal: usize,
        identity: usize,
        name: &syn::Ident,
        is_awaited: bool,
    ) -> bool {
        let index = &self.context.indexes[ordinal];
        index
            .implementations
            .iter()
            .filter(|implementation| {
                crate::module::util::owns_implementation(index, identity, implementation)
            })
            .any(|implementation| {
                implementation.syntax.items.iter().any(|member| {
                    matches!(member, syn::ImplItem::Fn(method)
                    if method.sig.ident == *name
                        && self.returns_workspace_result(
                            ordinal,
                            implementation.module,
                            &method.sig,
                            is_awaited,
                        ))
                })
            })
    }

    fn is_result_function_call(&self, call: &syn::ExprCall, is_awaited: bool) -> bool {
        let syn::Expr::Path(path) = &*call.func else {
            return false;
        };
        let is_free_function = self
            .context
            .resolve_workspace_path(self.ordinal, self.module, &path.path)
            .iter()
            .any(|target| {
                let Some(identity) = target.path.item else {
                    return false;
                };
                let item = &self.context.indexes[target.index].items[identity];
                matches!(&item.syntax, syn::Item::Fn(function)
                if self.returns_workspace_result(
                    target.index,
                    item.module,
                    &function.sig,
                    is_awaited,
                ))
            });
        if is_free_function || path.path.segments.len() < 2 {
            return is_free_function;
        }
        let Some(name) = path.path.segments.last().map(|segment| &segment.ident) else {
            return false;
        };
        let syn::Type::Path(owner) = Self::build_owner_type(&path.path) else {
            return false;
        };
        self.context
            .resolve_workspace_path(self.ordinal, self.module, &owner.path)
            .iter()
            .any(|target| {
                target.path.item.is_some_and(|identity| {
                    self.has_result_method(target.index, identity, name, is_awaited)
                })
            })
    }

    fn is_result_method_call(&self, call: &syn::ExprMethodCall, is_awaited: bool) -> bool {
        self.receiver_types(&call.receiver).iter().any(|receiver| {
            let mut typed = &receiver.typed;
            while let syn::Type::Reference(reference) = typed {
                typed = &reference.elem;
            }
            self.context
                .resolve_workspace_type(receiver.index, receiver.module, typed)
                .iter()
                .any(|target| {
                    target.path.item.is_some_and(|identity| {
                        self.has_result_method(target.index, identity, &call.method, is_awaited)
                    })
                })
        })
    }

    fn is_foreign_result_call(&self, call: &syn::ExprCall, is_awaited: bool) -> bool {
        let syn::Expr::Path(path) = &*call.func else {
            return false;
        };
        let functions = if is_awaited {
            [Self::TOKIO_FS_RESULT_FUNCTIONS, Self::TOKIO_SPAWN_FUNCTIONS].concat()
        } else {
            Self::STD_FS_RESULT_FUNCTIONS.to_vec()
        };
        self.context
            .resolve_external_call_paths(self.ordinal, self.module, &path.path)
            .iter()
            .any(|resolved| functions.contains(&resolved.format_path().as_str()))
    }

    fn is_awaited_join_handle(&self, base: &syn::Expr) -> bool {
        self.receiver_has(base, &["tokio::task::JoinHandle"])
    }

    fn is_workspace_result_source(&self, expression: &syn::Expr, is_awaited: bool) -> bool {
        match expression {
            syn::Expr::Call(call) => {
                self.is_result_function_call(call, is_awaited)
                    || self.is_foreign_result_call(call, is_awaited)
            }
            syn::Expr::MethodCall(call) => self.is_result_method_call(call, is_awaited),
            _ => is_awaited && self.is_awaited_join_handle(expression),
        }
    }

    fn is_workspace_result_call(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Paren(paren) => self.is_workspace_result_call(&paren.expr),
            syn::Expr::Group(group) => self.is_workspace_result_call(&group.expr),
            syn::Expr::Await(awaited) => self.is_workspace_result_source(&awaited.base, true),
            _ => self.is_workspace_result_source(expression, false),
        }
    }

    fn is_ok_of_workspace_result(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Paren(paren) => self.is_ok_of_workspace_result(&paren.expr),
            syn::Expr::Group(group) => self.is_ok_of_workspace_result(&group.expr),
            syn::Expr::MethodCall(call) => {
                call.method == "ok"
                    && call.args.is_empty()
                    && self.is_workspace_result_call(&call.receiver)
            }
            _ => false,
        }
    }

    fn stream_mentions(tokens: proc_macro2::TokenStream, name: &str) -> bool {
        tokens.into_iter().any(|tree| match tree {
            proc_macro2::TokenTree::Ident(ident) => ident == name,
            proc_macro2::TokenTree::Group(group) => Self::stream_mentions(group.stream(), name),
            proc_macro2::TokenTree::Literal(literal) => {
                syn::parse_str::<syn::LitStr>(&literal.to_string())
                    .is_ok_and(|text| names_format_argument(&text.value(), name))
            }
            _ => false,
        })
    }

    fn is_name_bound(pattern: &syn::Pat, name: &str) -> bool {
        match pattern {
            syn::Pat::Ident(binding) => {
                binding.ident == name
                    || binding
                        .subpat
                        .as_ref()
                        .is_some_and(|(_, inner)| Self::is_name_bound(inner, name))
            }
            syn::Pat::Type(typed) => Self::is_name_bound(&typed.pat, name),
            syn::Pat::Paren(paren) => Self::is_name_bound(&paren.pat, name),
            syn::Pat::Reference(reference) => Self::is_name_bound(&reference.pat, name),
            syn::Pat::Tuple(tuple) => tuple
                .elems
                .iter()
                .any(|element| Self::is_name_bound(element, name)),
            syn::Pat::TupleStruct(tuple) => tuple
                .elems
                .iter()
                .any(|element| Self::is_name_bound(element, name)),
            syn::Pat::Slice(slice) => slice
                .elems
                .iter()
                .any(|element| Self::is_name_bound(element, name)),
            syn::Pat::Or(alternatives) => alternatives
                .cases
                .iter()
                .any(|case| Self::is_name_bound(case, name)),
            syn::Pat::Struct(structure) => structure
                .fields
                .iter()
                .any(|field| Self::is_name_bound(&field.pat, name)),
            _ => false,
        }
    }

    fn find_shadowing_local<'syntax>(
        statement: &'syntax syn::Stmt,
        name: &str,
    ) -> Option<&'syntax syn::Local> {
        match statement {
            syn::Stmt::Local(local) => Self::is_name_bound(&local.pat, name).then_some(local),
            _ => None,
        }
    }

    fn is_read_later(following: &[syn::Stmt], name: &str) -> bool {
        for statement in following {
            let Some(shadow) = Self::find_shadowing_local(statement, name) else {
                if Self::stream_mentions(statement.to_token_stream(), name) {
                    return true;
                }
                continue;
            };
            return shadow
                .init
                .as_ref()
                .is_some_and(|init| Self::stream_mentions(init.expr.to_token_stream(), name));
        }
        false
    }

    fn check_unread_binding(&mut self, statement: &syn::Stmt, following: &[syn::Stmt]) {
        if self.rule != "ResultConsumption" {
            return;
        }
        let syn::Stmt::Local(local) = statement else {
            return;
        };
        let syn::Pat::Ident(binding) = &local.pat else {
            return;
        };
        let name = binding.ident.to_string();
        if name.len() < 2 || !name.starts_with('_') || binding.subpat.is_some() {
            return;
        }
        let Some(init) = &local.init else {
            return;
        };
        if init.diverge.is_some() || !self.is_workspace_result_call(&init.expr) {
            return;
        }
        if !Self::is_read_later(following, &name) {
            self.report_discarded_result(local.span());
        }
    }

    fn check_discarded_ok(&mut self, local: &syn::Local) {
        if self.rule != "ResultConsumption" || !matches!(local.pat, syn::Pat::Wild(_)) {
            return;
        }
        if expects_lint(&local.attrs, "let_underscore_must_use") {
            return;
        }
        let Some(init) = &local.init else {
            return;
        };
        if self.is_ok_of_workspace_result(&init.expr) {
            self.report_discarded_result(local.span());
        }
    }

    fn check_discarded_ok_statement(&mut self, statement: &syn::Stmt) {
        if self.rule != "ResultConsumption" {
            return;
        }
        let syn::Stmt::Expr(expression, Some(_)) = statement else {
            return;
        };
        if self.is_ok_of_workspace_result(expression) {
            self.report_discarded_result(expression.span());
        }
    }

    fn check_discarded_assignment(&mut self, expression: &syn::ExprAssign) {
        if self.rule != "ResultConsumption" || !matches!(&*expression.left, syn::Expr::Infer(_)) {
            return;
        }
        if self.is_workspace_result_call(&expression.right)
            || self.is_ok_of_workspace_result(&expression.right)
        {
            self.report_discarded_result(expression.span());
        }
    }

    fn is_lag_stopping_if(&self, expression: &syn::ExprIf) -> bool {
        if self.rule != "BroadcastLag" || self.loops == 0 {
            return false;
        }
        let syn::Expr::Let(condition) = &*expression.cond else {
            return false;
        };
        let Some((_, otherwise)) = &expression.else_branch else {
            return false;
        };
        self.lag_stopping_let(&condition.pat, &condition.expr) && Self::terminates_loop(otherwise)
    }

    fn bind_closure_state(&mut self, expression: &syn::ExprClosure) {
        let Some(state) = self.closure_state.take() else {
            return;
        };
        let Some(syn::Pat::Ident(name)) = expression.inputs.first() else {
            return;
        };
        self.bindings.push((name.ident.to_string(), state));
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

    fn any_owned_method(
        &self,
        ordinal: usize,
        identity: usize,
        name: &syn::Ident,
        predicate: &dyn Fn(usize, usize, &syn::Signature, &syn::Block) -> bool,
    ) -> bool {
        let index = &self.context.indexes[ordinal];
        index
            .implementations
            .iter()
            .filter(|implementation| {
                implementation.syntax.trait_.is_none()
                    && crate::module::util::owns_implementation(index, identity, implementation)
            })
            .any(|implementation| {
                implementation.syntax.items.iter().any(|member| {
                    matches!(member, syn::ImplItem::Fn(method)
                        if method.sig.ident == *name
                            && predicate(ordinal, implementation.module, &method.sig, &method.block))
                })
            })
    }

    fn resolve_self_targets(&self) -> Vec<crate::source::model::WorkspaceResolvedPath> {
        self.self_type
            .as_ref()
            .map(|typed| {
                self.context
                    .resolve_workspace_type(self.ordinal, self.module, typed)
            })
            .unwrap_or_default()
    }

    fn any_target_method(
        &self,
        targets: &[crate::source::model::WorkspaceResolvedPath],
        name: &syn::Ident,
        predicate: &dyn Fn(usize, usize, &syn::Signature, &syn::Block) -> bool,
    ) -> bool {
        targets.iter().any(|target| {
            target.path.item.is_some_and(|identity| {
                self.any_owned_method(target.index, identity, name, predicate)
            })
        })
    }

    fn any_called_function(
        &self,
        call: &syn::ExprCall,
        predicate: &dyn Fn(usize, usize, &syn::Signature, &syn::Block) -> bool,
    ) -> bool {
        let syn::Expr::Path(path) = &*call.func else {
            return false;
        };
        let targets = self
            .context
            .resolve_workspace_path(self.ordinal, self.module, &path.path);
        let is_free = targets.iter().any(|target| {
            target.path.item.is_some_and(|identity| {
                let item = &self.context.indexes[target.index].items[identity];
                matches!(&item.syntax, syn::Item::Fn(function)
                    if predicate(target.index, item.module, &function.sig, &function.block))
            })
        });
        let Some(name) = path.path.segments.last().map(|segment| &segment.ident) else {
            return is_free;
        };
        let syn::Type::Path(owner) = Self::build_owner_type(&path.path) else {
            return is_free;
        };
        let owner_targets = if owner.path.is_ident("Self") {
            self.resolve_self_targets()
        } else if path.path.segments.len() > 1 {
            self.context
                .resolve_workspace_path(self.ordinal, self.module, &owner.path)
        } else {
            Vec::new()
        };
        is_free || self.any_target_method(&owner_targets, name, predicate)
    }

    fn any_called_method(
        &self,
        call: &syn::ExprMethodCall,
        predicate: &dyn Fn(usize, usize, &syn::Signature, &syn::Block) -> bool,
    ) -> bool {
        if matches!(&*call.receiver, syn::Expr::Path(path) if path.path.is_ident("self")) {
            let targets = self.resolve_self_targets();
            return self.any_target_method(&targets, &call.method, predicate);
        }
        self.receiver_types(&call.receiver).iter().any(|receiver| {
            let mut typed = &receiver.typed;
            while let syn::Type::Reference(reference) = typed {
                typed = &reference.elem;
            }
            let targets =
                self.context
                    .resolve_workspace_type(receiver.index, receiver.module, typed);
            self.any_target_method(&targets, &call.method, predicate)
        })
    }

    fn any_workspace_callee(
        &self,
        future: &syn::Expr,
        predicate: &dyn Fn(usize, usize, &syn::Signature, &syn::Block) -> bool,
    ) -> bool {
        match future {
            syn::Expr::Paren(paren) => self.any_workspace_callee(&paren.expr, predicate),
            syn::Expr::Call(call) => self.any_called_function(call, predicate),
            syn::Expr::MethodCall(call) => self.any_called_method(call, predicate),
            _ => false,
        }
    }

    fn is_ready_future_path(path: &str) -> bool {
        let is_standard = matches!(path, "std::future::ready" | "core::future::ready");
        let is_futures = ["futures::future::", "futures_util::future::"]
            .iter()
            .find_map(|prefix| path.strip_prefix(prefix))
            .is_some_and(|name| matches!(name, "ready" | "ok" | "err"));
        is_standard || is_futures
    }

    fn is_immediate_future(&self, future: &syn::Expr) -> bool {
        if let syn::Expr::Async(block) = future {
            return match block.block.stmts.as_slice() {
                [syn::Stmt::Expr(syn::Expr::Await(awaited), None)] => {
                    self.is_immediate_future(&awaited.base)
                }
                _ => !AwaitVisitor::in_expression(future),
            };
        }
        if let syn::Expr::Call(call) = future
            && let syn::Expr::Path(path) = &*call.func
            && self
                .context
                .resolve_external_call_paths(self.ordinal, self.module, &path.path)
                .iter()
                .any(|path| Self::is_ready_future_path(&path.format_path()))
        {
            return true;
        }
        let awaits_nothing =
            |_: usize, _: usize, signature: &syn::Signature, block: &syn::Block| {
                signature.asyncness.is_some() && !block.stmts.iter().any(AwaitVisitor::in_statement)
            };
        if self.any_workspace_callee(future, &awaits_nothing) {
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

    fn invalid_timeout(&self, expression: &syn::ExprCall) -> bool {
        let syn::Expr::Path(path) = &*expression.func else {
            return false;
        };
        if !self
            .context
            .resolve_external_call_paths(self.ordinal, self.module, &path.path)
            .iter()
            .any(|path| {
                matches!(
                    path.format_path().as_str(),
                    "tokio::time::timeout" | "tokio::time::timeout_at"
                )
            })
        {
            return false;
        }
        expression
            .args
            .iter()
            .nth(1)
            .is_some_and(|future| self.is_immediate_future(future))
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

    fn is_awaited_broadcast_recv(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Paren(paren) => self.is_awaited_broadcast_recv(&paren.expr),
            syn::Expr::Group(group) => self.is_awaited_broadcast_recv(&group.expr),
            syn::Expr::Await(awaited) => !self.awaited_types(awaited).is_empty(),
            _ => false,
        }
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

    fn find_unfold_state_type(&self, call: &syn::ExprCall) -> Option<Rc<ResolvedTypeSyntax>> {
        let syn::Expr::Path(path) = &*call.func else {
            return None;
        };
        let unfolds = self
            .context
            .resolve_external_call_paths(self.ordinal, self.module, &path.path)
            .iter()
            .any(|path| {
                matches!(path.package.as_str(), "futures" | "futures_util")
                    && path.segments.last().is_some_and(|name| name == "unfold")
            });
        if !unfolds {
            return None;
        }
        self.receiver_types(call.args.first()?).into_iter().next()
    }

    fn save_boundary(&self) -> VisitorBoundary {
        VisitorBoundary {
            synchronous_boundary: self.synchronous_boundary,
            in_drop: self.in_drop,
            loops: self.loops,
            async_context: self.async_context,
        }
    }

    fn restore_boundary(&mut self, saved: VisitorBoundary) {
        self.synchronous_boundary = saved.synchronous_boundary;
        self.in_drop = saved.in_drop;
        self.loops = saved.loops;
        self.async_context = saved.async_context;
    }

    fn visit_closure_argument(&mut self, argument: &syn::Expr, inherit: bool) {
        self.inherit_async = inherit && matches!(argument, syn::Expr::Closure(_));
        self.visit_expr(argument);
        self.inherit_async = false;
    }

    fn is_runtime_thread_hook(name: &syn::Ident) -> bool {
        name == "on_enable" || name == "on_reload"
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

    fn check_temporary_guard(&mut self, statement: &syn::Stmt, is_tail: bool) {
        let temporary = match statement {
            syn::Stmt::Expr(expression, Some(_)) => Some(expression),
            syn::Stmt::Expr(expression, None) if is_tail => Some(expression),
            syn::Stmt::Local(local) => local
                .init
                .as_ref()
                .map(|init| &*init.expr)
                .filter(|expression| !matches!(expression, syn::Expr::Await(_))),
            _ => None,
        };
        if self.rule == "LockChoice"
            && temporary.is_some_and(|expression| self.temporary_guard_without_await(expression))
        {
            self.report(statement.span(), Self::ASYNC_LOCK_WITHOUT_AWAIT_MESSAGE);
        }
    }

    fn find_guard_call<'call>(
        &self,
        call: &'call syn::ExprMethodCall,
    ) -> &'call syn::ExprMethodCall {
        if matches!(
            call.method.to_string().as_str(),
            "unwrap" | "unwrap_or_else" | "expect"
        ) && let syn::Expr::MethodCall(lock) = &*call.receiver
            && matches!(lock.method.to_string().as_str(), "lock" | "write")
            && self.receiver_has(&lock.receiver, &["std::sync::Mutex", "std::sync::RwLock"])
        {
            return lock;
        }
        call
    }

    fn check_guard_local(&mut self, statement: &syn::Stmt, following: &[syn::Stmt]) {
        let syn::Stmt::Local(local) = statement else {
            return;
        };
        let Some(name) = Self::guard_binding(&local.pat) else {
            return;
        };
        let Some(init) = &local.init else {
            return;
        };
        let base = match &*init.expr {
            syn::Expr::Await(awaited) => &*awaited.base,
            expression => expression,
        };
        let syn::Expr::MethodCall(call) = base else {
            return;
        };
        let call = self.find_guard_call(call);
        if self.is_guard_method(call) && self.receiver_has(&call.receiver, Self::GUARDED_LOCKS) {
            self.check_guard_tail(local, call, &name.to_string(), following);
        }
    }

    const GUARDED_LOCKS: &'static [&'static str] = &[
        "tokio::sync::Mutex",
        "tokio::sync::RwLock",
        "parking_lot::Mutex",
        "parking_lot::RwLock",
        "std::sync::Mutex",
        "std::sync::RwLock",
    ];

    const FILE_BLOCKING_METHODS: &'static [&'static str] = &[
        "read",
        "read_to_end",
        "read_to_string",
        "read_exact",
        "read_vectored",
        "read_buf",
        "bytes",
        "write",
        "write_all",
        "write_vectored",
        "write_fmt",
        "flush",
        "seek",
        "rewind",
        "stream_position",
        "seek_relative",
        "sync_all",
        "sync_data",
        "set_len",
        "metadata",
        "try_clone",
        "set_permissions",
    ];

    const TCP_STREAM_IO_METHODS: &'static [&'static str] = &[
        "read",
        "read_exact",
        "read_to_end",
        "read_to_string",
        "write",
        "write_all",
        "flush",
        "peek",
        "shutdown",
    ];

    const COMMAND_IO_METHODS: &'static [&'static str] = &["spawn", "output", "status"];

    const STD_IO_FUNCTIONS: &'static [&'static str] = &[
        "std::io::stdin",
        "std::io::stdout",
        "std::io::stderr",
        "std::io::copy",
        "std::io::read_to_string",
    ];

    const STD_FS_RESULT_FUNCTIONS: &'static [&'static str] = &[
        "std::fs::canonicalize",
        "std::fs::copy",
        "std::fs::create_dir",
        "std::fs::create_dir_all",
        "std::fs::exists",
        "std::fs::hard_link",
        "std::fs::metadata",
        "std::fs::read",
        "std::fs::read_dir",
        "std::fs::read_link",
        "std::fs::read_to_string",
        "std::fs::remove_dir",
        "std::fs::remove_dir_all",
        "std::fs::remove_file",
        "std::fs::rename",
        "std::fs::set_permissions",
        "std::fs::symlink_metadata",
        "std::fs::write",
    ];

    const TOKIO_FS_RESULT_FUNCTIONS: &'static [&'static str] = &[
        "tokio::fs::canonicalize",
        "tokio::fs::copy",
        "tokio::fs::create_dir",
        "tokio::fs::create_dir_all",
        "tokio::fs::hard_link",
        "tokio::fs::metadata",
        "tokio::fs::read",
        "tokio::fs::read_dir",
        "tokio::fs::read_link",
        "tokio::fs::read_to_string",
        "tokio::fs::remove_dir",
        "tokio::fs::remove_dir_all",
        "tokio::fs::remove_file",
        "tokio::fs::rename",
        "tokio::fs::set_permissions",
        "tokio::fs::symlink_metadata",
        "tokio::fs::try_exists",
        "tokio::fs::write",
    ];

    const TOKIO_SPAWN_FUNCTIONS: &'static [&'static str] = &[
        "tokio::spawn",
        "tokio::task::spawn",
        "tokio::task::spawn_blocking",
        "tokio::task::spawn_local",
    ];

    const CHANNEL_ENDPOINT_TYPES: &'static [&'static str] = &[
        "tokio::sync::watch::Sender",
        "tokio::sync::watch::Receiver",
        "tokio::sync::mpsc::Sender",
        "tokio::sync::mpsc::Receiver",
        "tokio::sync::mpsc::UnboundedSender",
        "tokio::sync::mpsc::UnboundedReceiver",
        "tokio::sync::broadcast::Sender",
        "tokio::sync::broadcast::Receiver",
        "tokio::sync::oneshot::Sender",
        "tokio::sync::oneshot::Receiver",
        "std::sync::mpsc::Sender",
        "std::sync::mpsc::SyncSender",
        "std::sync::mpsc::Receiver",
    ];

    const LOG_EVENT_MACROS: &'static [&'static str] = &[
        "tracing::trace",
        "tracing::debug",
        "tracing::info",
        "tracing::warn",
        "tracing::error",
        "tracing::event",
        "log::trace",
        "log::debug",
        "log::info",
        "log::warn",
        "log::error",
    ];

    const DISCARDED_RESULT_MESSAGE: &'static str = "Handle or propagate the Result instead of dropping it through `_ =`, `.ok()` or an unread `_name` binding (error-handling.rs.md)";

    const UNPROVEN_DISCARD_MESSAGE: &'static str = "The discard expectation requires a proven oneshot::Sender::send, mpsc::UnboundedSender::send, broadcast::Sender::send or mpsc::Sender::try_send; an unresolved receiver cannot establish this exception (error-handling.rs.md)";

    const JOIN_SET_BEHIND_LOCK_MESSAGE: &'static str = "A JoinSet behind a lock keeps every finished task until join_next runs: spawn on a TaskScope, whose drain owns the JoinSet (async.rs.md, Tasks)";

    const CRITICAL_SECTION_WRITE_MESSAGE: &'static str = "A critical section computes first and writes once: build the new value with a pure rule, then store it in the last statement of the guard's scope (async.rs.md, Shared state and locks)";

    const LOCK_AROUND_PRIMITIVE_MESSAGE: &'static str = "A counter or a flag is an atomic, not a lock around a bool or an integer (async.rs.md, Shared state and locks)";

    const ASYNC_LOCK_WITHOUT_AWAIT_MESSAGE: &'static str = "A lock whose guard no .await crosses is parking_lot::Mutex or RwLock; tokio::sync::Mutex is only for a guard that lives across an .await (async.rs.md, Shared state and locks)";

    const BROADCAST_LAG_MESSAGE: &'static str = "Match RecvError::Lagged and go on: a loop that ends at the first lag leaves events in the channel (no-callbacks.rs.md, Things that happen)";

    const SELECT_CANCEL_SAFETY_MESSAGE: &'static str = "In a looped select! every branch is cancel safe, or its future is created once before the loop and polled through &mut (async.rs.md, Tasks)";

    const IMPURE_UPDATE_MESSAGE: &'static str = "The closure of send_modify, send_if_modified and fetch_update computes the new value from the current one and does nothing else: no I/O, no lock, no other channel, no captured write (async.rs.md)";

    const BLOCKING_LOCK_MESSAGE: &'static str = "A synchronous Drop, on_enable or on_reload on a runtime thread cannot call blocking_lock: use a parking_lot lock for data no .await crosses (async.rs.md, Shared state and locks)";

    const BLOCKING_CHANNEL_MESSAGE: &'static str = "Drop does not block on a channel: end the task by dropping its sender and cancel through a guard, not through recv or blocking_send (no-callbacks.rs.md, Cleanup)";

    const WATCH_TIMEOUT_MESSAGE: &'static str = "A timeout wraps a call that can actually wait; around a read that returns at once it never fires: read the watch with borrow() (async.rs.md)";

    const DURATION_OVERLOAD_MESSAGE: &'static str = "Pass the Duration itself where the API takes one: keep the constant a Duration and convert to an integer only at a call that takes only an integer (time-api.rs.md)";
}

impl<'ast> Visit<'ast> for TypedRuntimeVisitor<'_> {
    fn visit_expr_match(&mut self, expression: &'ast syn::ExprMatch) {
        if self.rule == "BroadcastLag" && self.loops > 0 && self.broadcast_recv(&expression.expr) {
            for arm in &expression.arms {
                if self.lag_pattern(&arm.pat) && Self::terminates_loop(&arm.body) {
                    self.report(arm.span(), Self::BROADCAST_LAG_MESSAGE);
                }
            }
        }
        self.visit_expr(&expression.expr);
        for arm in &expression.arms {
            let length = self.bindings.len();
            self.bind_pattern(&arm.pat);
            self.bind_payload(&arm.pat, &expression.expr);
            syn::visit::visit_arm(self, arm);
            self.bindings.truncate(length);
        }
    }
    fn visit_expr_closure(&mut self, expression: &'ast syn::ExprClosure) {
        let outer = self.save_boundary();
        let inherited = std::mem::take(&mut self.inherit_async);
        self.async_context = expression.asyncness.is_some() || inherited;
        let length = self.bindings.len();
        self.synchronous_boundary = false;
        self.in_drop = false;
        self.loops = 0;
        for pattern in &expression.inputs {
            self.bind_pattern(pattern);
        }
        self.bind_closure_state(expression);
        syn::visit::visit_expr_closure(self, expression);
        self.bindings.truncate(length);
        self.restore_boundary(outer);
    }

    fn visit_expr_async(&mut self, expression: &'ast syn::ExprAsync) {
        let outer = self.save_boundary();
        self.async_context = true;
        self.synchronous_boundary = false;
        self.in_drop = false;
        self.loops = 0;
        syn::visit::visit_expr_async(self, expression);
        self.restore_boundary(outer);
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
            let awaits_unsafe_callee =
                self.any_workspace_callee(&future, &|ordinal, module, signature, block| {
                    self.awaits_unsafe_call(ordinal, module, signature, block)
                });
            if qualified
                || awaits_unsafe_callee
                || effects
                    .methods
                    .iter()
                    .any(|call| self.unsafe_select_call(call))
            {
                self.report(future.span(), Self::SELECT_CANCEL_SAFETY_MESSAGE);
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
        self.generic_scopes.push(item.generics.clone());
        syn::visit::visit_item_impl(self, item);
        self.generic_scopes.pop();
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
        let outer = self.save_boundary();
        self.async_context = item.sig.asyncness.is_some();
        let expectation = self.thread_expect;
        self.thread_expect |= expects_lint(&item.attrs, "disallowed_methods");
        let parent = std::mem::take(&mut self.bindings);
        self.synchronous_boundary = false;
        self.in_drop = false;
        self.bind_signature(&item.sig);
        self.generic_scopes.push(item.sig.generics.clone());
        syn::visit::visit_item_fn(self, item);
        self.generic_scopes.pop();
        self.bindings = parent;
        self.restore_boundary(outer);
        self.thread_expect = expectation;
        self.runtime_main = runtime_main;
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        let outer = self.save_boundary();
        self.async_context = item.sig.asyncness.is_some();
        let expectation = self.thread_expect;
        self.thread_expect |= expects_lint(&item.attrs, "disallowed_methods");
        let parent = std::mem::take(&mut self.bindings);
        let is_synchronous = item.sig.asyncness.is_none();
        let is_drop = is_synchronous && item.sig.ident == "drop";
        let is_hook = is_synchronous && Self::is_runtime_thread_hook(&item.sig.ident);
        self.in_drop = is_drop;
        self.synchronous_boundary = is_drop || is_hook;
        self.bind_signature(&item.sig);
        self.generic_scopes.push(item.sig.generics.clone());
        syn::visit::visit_impl_item_fn(self, item);
        self.generic_scopes.pop();
        self.bindings = parent;
        self.restore_boundary(outer);
        self.thread_expect = expectation;
    }

    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        let bindings = std::mem::take(&mut self.bindings);
        let outer = self.save_boundary();
        self.synchronous_boundary =
            item.sig.asyncness.is_none() && Self::is_runtime_thread_hook(&item.sig.ident);
        self.in_drop = false;
        self.async_context = item.sig.asyncness.is_some();
        self.bind_signature(&item.sig);
        syn::visit::visit_trait_item_fn(self, item);
        self.bindings = bindings;
        self.restore_boundary(outer);
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        let length = self.bindings.len();
        for (position, statement) in block.stmts.iter().enumerate() {
            self.visit_stmt(statement);
            self.check_unread_binding(statement, &block.stmts[position + 1..]);
            if !matches!(self.rule, "CriticalSectionWrite" | "LockChoice") {
                continue;
            }
            let is_tail = position + 1 == block.stmts.len();
            self.check_temporary_guard(statement, is_tail);
            self.check_guard_local(statement, &block.stmts[position + 1..]);
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
            self.report(local.span(), Self::UNPROVEN_DISCARD_MESSAGE);
        }
        if self.rule == "ResultConsumption"
            && let syn::Pat::Ident(name) = &local.pat
            && name.ident.to_string().starts_with('_')
            && let Some(init) = &local.init
            && self.is_channel_send(&init.expr)
        {
            self.report(local.span(), Self::UNPROVEN_DISCARD_MESSAGE);
        }
        self.check_discarded_ok(local);
        if self.rule == "BroadcastLag"
            && self.loops > 0
            && let Some(init) = &local.init
            && let Some((_, diverge)) = &init.diverge
            && self.lag_stopping_let(&local.pat, &init.expr)
            && Self::terminates_loop(diverge)
        {
            self.report(local.span(), Self::BROADCAST_LAG_MESSAGE);
        }
        syn::visit::visit_local(self, local);
        self.bind_initializer(local);
        self.thread_expect = expectation;
    }

    fn visit_stmt(&mut self, statement: &'ast syn::Stmt) {
        self.check_discarded_ok_statement(statement);
        syn::visit::visit_stmt(self, statement);
    }

    fn visit_expr_assign(&mut self, expression: &'ast syn::ExprAssign) {
        self.check_discarded_assignment(expression);
        syn::visit::visit_expr_assign(self, expression);
    }

    fn visit_expr_method_call(&mut self, expression: &'ast syn::ExprMethodCall) {
        let expectation = self.thread_expect;
        self.thread_expect |= expects_lint(&expression.attrs, "disallowed_methods");
        let index = &self.context.indexes[self.ordinal];
        let in_test = index.modules[self.module].tree != crate::source::model::SourceTree::Src;
        if self.rule == "BlockOnInTest" && in_test && self.is_local_pool_run(expression) {
            self.report(expression.span(), RuntimeVisitor::BLOCK_ON_IN_TEST_MESSAGE);
        }
        if expression.method == "block_on" && self.is_block_on_receiver(&expression.receiver) {
            let outside_boundary = !self.runtime_main
                && !crate::declaration::util::is_technology(self.context, index, self.module);
            if self.rule == "BlockOnInTest" && in_test {
                self.report(expression.span(), RuntimeVisitor::BLOCK_ON_IN_TEST_MESSAGE);
            }
            if self.rule == "BlockOnOutsideBoundary" && outside_boundary {
                self.report(
                    expression.span(),
                    RuntimeVisitor::BLOCK_ON_OUTSIDE_BOUNDARY_MESSAGE,
                );
            }
        }
        let blocking_async = self.rule == "BlockingCallInAsync" && self.async_context;
        if blocking_async && self.is_blocking_method(expression) {
            self.report(expression.span(), TaskVisitor::BLOCKING_CALL_MESSAGE);
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
            self.report(
                expression.span(),
                TaskVisitor::THREAD_OUTSIDE_TECHNOLOGY_MESSAGE,
            );
        }
        if self.rule == "ImpureUpdateClosure" {
            self.check_update(expression);
        }
        if self.rule == "BlockingLockInSync" {
            self.check_blocking_in_sync(expression);
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
            self.report(expression.span(), Self::BROADCAST_LAG_MESSAGE);
        }
        let length = self.bindings.len();
        syn::visit::visit_expr_while(self, expression);
        self.bindings.truncate(length);
        self.loops -= 1;
    }

    fn visit_expr_if(&mut self, expression: &'ast syn::ExprIf) {
        if self.is_lag_stopping_if(expression) {
            self.report(expression.span(), Self::BROADCAST_LAG_MESSAGE);
        }
        let length = self.bindings.len();
        self.visit_expr(&expression.cond);
        self.visit_block(&expression.then_branch);
        self.bindings.truncate(length);
        if let Some((_, otherwise)) = &expression.else_branch {
            self.visit_expr(otherwise);
        }
    }

    fn visit_expr_try(&mut self, expression: &'ast syn::ExprTry) {
        let is_in_lag_loop = self.rule == "BroadcastLag" && self.loops > 0;
        if is_in_lag_loop && self.is_awaited_broadcast_recv(&expression.expr) {
            self.report(expression.span(), Self::BROADCAST_LAG_MESSAGE);
        }
        syn::visit::visit_expr_try(self, expression);
    }

    fn visit_expr_let(&mut self, expression: &'ast syn::ExprLet) {
        self.visit_expr(&expression.expr);
        if !self.bind_payload(&expression.pat, &expression.expr) {
            self.bind_pattern(&expression.pat);
        }
    }

    fn visit_expr_call(&mut self, expression: &'ast syn::ExprCall) {
        if self.rule == "WatchTimeout" && self.invalid_timeout(expression) {
            self.report(expression.span(), Self::WATCH_TIMEOUT_MESSAGE);
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
            self.report(expression.span(), Self::DURATION_OVERLOAD_MESSAGE);
        }
        self.visit_expr(&expression.func);
        let inherit = self.async_context && !self.runs_off_async_call(expression);
        let state = self.find_unfold_state_type(expression);
        for (position, argument) in expression.args.iter().enumerate() {
            if position == 1 && matches!(argument, syn::Expr::Closure(_)) {
                self.closure_state.clone_from(&state);
            }
            self.visit_closure_argument(argument, inherit);
        }
        self.closure_state = None;
    }
}

struct VisitorBoundary {
    synchronous_boundary: bool,
    in_drop: bool,
    loops: usize,
    async_context: bool,
}
