use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::{build_node_finding, format_module_path};
use crate::config::model::Role;
use crate::declaration::util::is_technology;
use crate::module::util::find_kind_name;
use crate::runtime::util::names_format_argument;
use crate::runtime::util::node_predicate;
use crate::source::model::Resolution;
use crate::source::util::find_node_span;
use crate::source::util::list_derive_paths;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct TimePrimitive;

struct TimeVisitor<'context> {
    context: &'context RuleContext,
    ordinal: usize,
    module: usize,
    violations: Vec<TimeViolation>,
    bindings: Vec<LocalBinding>,
    in_serde_entry_type: bool,
    in_trait_impl: bool,
}

impl TimeVisitor<'_> {
    fn report(&mut self, span: proc_macro2::Span, message: &'static str) {
        self.violations.push(TimeViolation { span, message });
    }

    fn report_instant(&mut self, span: proc_macro2::Span) {
        self.report(
            span,
            "An Instant is not a field or a return type: keep it a local and keep the elapsed Duration (time-api.rs.md)",
        );
    }

    fn report_used_instant(&mut self, span: proc_macro2::Span) {
        self.report(
            span,
            "An Instant is never sent, stored or shown: pass elapsed() (time-api.rs.md)",
        );
    }

    fn report_unit_suffix(&mut self, span: proc_macro2::Span) {
        self.report(
            span,
            "A duration is a Duration, not an integer or text with a unit suffix (time-api.rs.md)",
        );
    }

    fn bind_signature(&mut self, signature: &syn::Signature) {
        self.bindings.clear();
        for parameter in &signature.inputs {
            let syn::FnArg::Typed(parameter) = parameter else {
                continue;
            };
            self.bind_pattern(&parameter.pat, Some(&parameter.ty), false);
        }
    }

    fn sender_type(&self, typed: &syn::Type) -> bool {
        let typed = match typed {
            syn::Type::Reference(reference) => &*reference.elem,
            typed => typed,
        };
        self.context
            .resolve_outer_external_type_paths(self.ordinal, self.module, typed)
            .iter()
            .any(|path| {
                [
                    "tokio::sync::mpsc::Sender",
                    "tokio::sync::mpsc::UnboundedSender",
                    "tokio::sync::oneshot::Sender",
                    "tokio::sync::broadcast::Sender",
                    "tokio::sync::watch::Sender",
                    "std::sync::mpsc::Sender",
                    "std::sync::mpsc::SyncSender",
                ]
                .contains(&path.format_path().as_str())
            })
    }

    fn bind_pattern(&mut self, pattern: &syn::Pat, typed: Option<&syn::Type>, instant: bool) {
        match pattern {
            syn::Pat::Ident(name) => {
                self.bindings.push(LocalBinding {
                    name: name.ident.to_string(),
                    is_instant: instant || typed.is_some_and(|typed| self.instant(typed)),
                    is_sender: typed.is_some_and(|typed| self.sender_type(typed)),
                });
                if let Some((_, pattern)) = &name.subpat {
                    self.bind_pattern(pattern, None, false);
                }
            }
            syn::Pat::Type(pattern) => self.bind_pattern(&pattern.pat, Some(&pattern.ty), instant),
            syn::Pat::Paren(pattern) => self.bind_pattern(&pattern.pat, typed, instant),
            syn::Pat::Reference(pattern) => self.bind_pattern(&pattern.pat, None, false),
            syn::Pat::Tuple(pattern) => {
                for pattern in &pattern.elems {
                    self.bind_pattern(pattern, None, false);
                }
            }
            syn::Pat::TupleStruct(pattern) => {
                for pattern in &pattern.elems {
                    self.bind_pattern(pattern, None, false);
                }
            }
            syn::Pat::Slice(pattern) => {
                for pattern in &pattern.elems {
                    self.bind_pattern(pattern, None, false);
                }
            }
            syn::Pat::Struct(pattern) => {
                for field in &pattern.fields {
                    self.bind_pattern(&field.pat, None, false);
                }
            }
            syn::Pat::Or(pattern) => {
                for pattern in &pattern.cases {
                    self.bind_pattern(pattern, None, false);
                }
            }
            _ => syn::visit::visit_pat(self, pattern),
        }
    }

    fn visit_condition(&mut self, condition: &syn::Expr) {
        match condition {
            syn::Expr::Let(condition) => {
                self.visit_expr(&condition.expr);
                let instant = self.local_instant(&condition.expr);
                self.bind_pattern(&condition.pat, None, instant);
            }
            syn::Expr::Binary(binary) if matches!(binary.op, syn::BinOp::And(_)) => {
                self.visit_condition(&binary.left);
                self.visit_condition(&binary.right);
            }
            _ => self.visit_expr(condition),
        }
    }

    fn instant(&self, typed: &syn::Type) -> bool {
        let index = &self.context.indexes[self.ordinal];
        let cfg = node_predicate(
            index,
            self.module,
            &index.modules[self.module].file,
            find_node_span(typed.span()),
        );
        self.context
            .resolve_external_type_paths_under(self.ordinal, self.module, typed, &cfg)
            .iter()
            .any(|path| {
                matches!(
                    path.format_path().as_str(),
                    "std::time::Instant" | "tokio::time::Instant"
                )
            })
    }

    fn primitive(&self, typed: &syn::Type) -> bool {
        if self
            .context
            .resolve_external_type_paths(self.ordinal, self.module, typed)
            .iter()
            .any(|path| {
                matches!(
                    path.format_path().as_str(),
                    "std::string::String" | "alloc::string::String"
                )
            })
        {
            return true;
        }
        match typed {
            syn::Type::Reference(reference) => return self.primitive(&reference.elem),
            syn::Type::Array(array) => return self.primitive(&array.elem),
            syn::Type::Slice(slice) => return self.primitive(&slice.elem),
            _ => {}
        }
        self.context.resolve_type_syntax(self.ordinal, self.module, typed).iter().any(|resolved|
            matches!(&resolved.typed, syn::Type::Path(path) if path.path.get_ident().is_some_and(|name|
                ["u8","u16","u32","u64","u128","usize","i8","i16","i32","i64","i128","isize","f32","f64","str"].contains(&name.to_string().as_str()))))
    }

    fn is_wire_boundary(&self) -> bool {
        let index = &self.context.indexes[self.ordinal];
        let is_contract = self
            .context
            .find_role_config(&index.crate_name)
            .is_some_and(|config| config.role == Some(Role::Contract));
        is_contract || is_technology(self.context, index, self.module)
    }

    fn derives_serde(&self, attributes: &[syn::Attribute]) -> bool {
        let index = &self.context.indexes[self.ordinal];
        list_derive_paths(attributes).iter().any(|path| {
            matches!(index.resolve_path(self.module, path), Resolution::External(resolved)
            if matches!(
                resolved.as_str(),
                "serde::Serialize"
                    | "serde::Deserialize"
                    | "serde_derive::Serialize"
                    | "serde_derive::Deserialize"
            ))
        })
    }

    fn is_under_model(&self) -> bool {
        let index = &self.context.indexes[self.ordinal];
        let mut current = Some(self.module);
        while let Some(module) = current {
            if index.modules[module].module.path.last().map(String::as_str) == Some("model") {
                return true;
            }
            current = index.modules[module].parent;
        }
        false
    }

    fn is_serde_entry_type(&self, attributes: &[syn::Attribute]) -> bool {
        let crate_name = &self.context.indexes[self.ordinal].crate_name;
        let kind = find_kind_name(self.context, self.ordinal, self.module);
        self.context.is_entry_kind(crate_name, kind)
            && !self.is_under_model()
            && self.derives_serde(attributes)
    }

    fn has_unit_suffix(ident: &syn::Ident) -> bool {
        let name = ident.to_string().to_uppercase();
        [
            "_MS",
            "_MILLIS",
            "_SECS",
            "_SECONDS",
            "_NANOS",
            "_MICROS",
            "_MINUTES",
            "_HOURS",
            "_DAYS",
            "_WEEKS",
            "_MILLISECONDS",
            "_MICROSECONDS",
            "_NANOSECONDS",
            "_SEC",
            "_MINS",
        ]
        .iter()
        .any(|suffix| name.ends_with(suffix))
    }

    fn wrapped_primitive(&self, typed: &syn::Type) -> bool {
        let syn::Type::Path(path) = typed else {
            return false;
        };
        let is_wrapper = self
            .context
            .resolve_outer_external_type_paths(self.ordinal, self.module, typed)
            .iter()
            .any(|outer| {
                ["std", "core", "alloc"].contains(&outer.package.as_str())
                    && outer.segments.last().is_some_and(|name| {
                        ["Option", "Vec", "VecDeque", "Box"].contains(&name.as_str())
                    })
            });
        let Some(syn::PathArguments::AngleBracketed(arguments)) =
            path.path.segments.last().map(|segment| &segment.arguments)
        else {
            return false;
        };
        is_wrapper
            && matches!(arguments.args.first(), Some(syn::GenericArgument::Type(inner))
                if arguments.args.len() == 1 && self.primitive(inner))
    }

    fn is_unit_named_primitive(&self, ident: &syn::Ident, typed: &syn::Type) -> bool {
        Self::has_unit_suffix(ident) && (self.primitive(typed) || self.wrapped_primitive(typed))
    }

    fn returns_duration(&self, signature: &syn::Signature) -> bool {
        let syn::ReturnType::Type(_, returned) = &signature.output else {
            return false;
        };
        self.context
            .resolve_outer_external_type_paths(self.ordinal, self.module, returned)
            .iter()
            .any(|path| {
                ["std", "core"].contains(&path.package.as_str())
                    && path.segments.last().is_some_and(|name| name == "Duration")
            })
    }

    fn check_parameters(&mut self, signature: &syn::Signature) {
        if self.is_wire_boundary() || self.returns_duration(signature) {
            return;
        }
        for parameter in &signature.inputs {
            let syn::FnArg::Typed(parameter) = parameter else {
                continue;
            };
            let syn::Pat::Ident(binding) = parameter.pat.as_ref() else {
                continue;
            };
            if self.is_unit_named_primitive(&binding.ident, &parameter.ty) {
                self.report_unit_suffix(parameter.span());
            }
        }
    }

    fn check_literal_local(&mut self, local: &syn::Local) {
        let binding = match &local.pat {
            syn::Pat::Ident(binding) => binding,
            syn::Pat::Type(typed) => match typed.pat.as_ref() {
                syn::Pat::Ident(binding) => binding,
                _ => return,
            },
            _ => return,
        };
        let integer_literal = local.init.as_ref().is_some_and(|init| {
            matches!(init.expr.as_ref(), syn::Expr::Lit(literal) if matches!(literal.lit, syn::Lit::Int(_)))
        });
        if integer_literal && Self::has_unit_suffix(&binding.ident) && !self.is_wire_boundary() {
            self.report_unit_suffix(local.span());
        }
    }

    fn local_instant(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Path(path) => self
                .bindings
                .iter()
                .rev()
                .find(|binding| path.path.is_ident(&binding.name))
                .is_some_and(|binding| binding.is_instant),
            syn::Expr::Reference(reference) => self.local_instant(&reference.expr),
            syn::Expr::Paren(paren) => self.local_instant(&paren.expr),
            syn::Expr::Cast(cast) => self.local_instant(&cast.expr),
            syn::Expr::Call(call) => {
                matches!(&*call.func, syn::Expr::Path(path) if self.context.resolve_external_call_paths(self.ordinal, self.module, &path.path).iter().any(|path| matches!(path.format_path().as_str(), "std::time::Instant::now" | "tokio::time::Instant::now")))
            }
            _ => false,
        }
    }

    fn logged_instant(&self, field: &[proc_macro2::TokenTree]) -> bool {
        let assignment = field.iter().position(
            |token| matches!(token, proc_macro2::TokenTree::Punct(punct) if punct.as_char() == '='),
        );
        let mut value = &field[assignment.map_or(0, |position| position + 1)..];
        if matches!(value.first(), Some(proc_macro2::TokenTree::Punct(punct)) if matches!(punct.as_char(), '?' | '%'))
        {
            value = &value[1..];
        }
        let tokens = value.iter().cloned().collect::<proc_macro2::TokenStream>();
        let Ok(expression) = syn::parse2::<syn::Expr>(tokens) else {
            return false;
        };
        if self.local_instant(&expression) {
            return true;
        }
        let syn::Expr::Lit(literal) = expression else {
            return false;
        };
        let syn::Lit::Str(message) = literal.lit else {
            return false;
        };
        self.bindings.iter().any(|candidate| {
            self.bindings
                .iter()
                .rev()
                .find(|binding| binding.name == candidate.name)
                .is_some_and(|binding| binding.is_instant)
                && names_format_argument(&message.value(), &candidate.name)
        })
    }

    fn is_channel_constructor(&self, expression: &syn::Expr) -> bool {
        let syn::Expr::Call(call) = expression else {
            return false;
        };
        let syn::Expr::Path(path) = &*call.func else {
            return false;
        };
        self.context
            .resolve_external_call_paths(self.ordinal, self.module, &path.path)
            .iter()
            .any(|path| {
                [
                    "tokio::sync::mpsc::channel",
                    "tokio::sync::mpsc::unbounded_channel",
                    "tokio::sync::oneshot::channel",
                    "tokio::sync::broadcast::channel",
                    "tokio::sync::watch::channel",
                    "std::sync::mpsc::channel",
                    "std::sync::mpsc::sync_channel",
                ]
                .contains(&path.format_path().as_str())
            })
    }

    fn mark_channel_sender(&mut self, local: &syn::Local) {
        let syn::Pat::Tuple(pattern) = &local.pat else {
            return;
        };
        let Some(syn::Pat::Ident(sender)) = pattern.elems.first() else {
            return;
        };
        if !local
            .init
            .as_ref()
            .is_some_and(|init| self.is_channel_constructor(&init.expr))
        {
            return;
        }
        let name = sender.ident.to_string();
        if let Some(binding) = self
            .bindings
            .iter_mut()
            .rev()
            .find(|binding| binding.name == name)
        {
            binding.is_sender = true;
        }
    }

    fn is_formatting_macro(&self, path: &syn::Path) -> bool {
        const NAMES: [&str; 4] = ["format", "format_args", "write", "writeln"];
        let prelude = path
            .get_ident()
            .is_some_and(|name| NAMES.iter().any(|candidate| name == candidate))
            && matches!(
                self.context.indexes[self.ordinal].resolve_path(self.module, path),
                Resolution::Local | Resolution::Unresolved
            );
        prelude
            || self
                .context
                .resolve_external_call_paths(self.ordinal, self.module, path)
                .iter()
                .any(|path| {
                    matches!(path.package.as_str(), "std" | "core" | "alloc")
                        && path
                            .segments
                            .last()
                            .is_some_and(|name| NAMES.contains(&name.as_str()))
                })
    }

    fn logs_instant(&self, tokens: proc_macro2::TokenStream) -> bool {
        let mut field = Vec::new();
        for token in tokens
            .into_iter()
            .chain(std::iter::once(proc_macro2::TokenTree::Punct(
                proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone),
            )))
        {
            if matches!(&token, proc_macro2::TokenTree::Punct(punct) if matches!(punct.as_char(), ',' | ';'))
            {
                if self.logged_instant(&field) {
                    return true;
                }
                field.clear();
            } else {
                field.push(token);
            }
        }
        false
    }
}

impl<'ast> Visit<'ast> for TimeVisitor<'_> {
    fn visit_pat_ident(&mut self, pattern: &'ast syn::PatIdent) {
        self.bind_pattern(&syn::Pat::Ident(pattern.clone()), None, false);
    }

    fn visit_signature(&mut self, signature: &'ast syn::Signature) {
        self.visit_return_type(&signature.output);
    }
    fn visit_expr_closure(&mut self, expression: &'ast syn::ExprClosure) {
        let length = self.bindings.len();
        for parameter in &expression.inputs {
            self.bind_pattern(parameter, None, false);
        }
        self.visit_expr(&expression.body);
        self.bindings.truncate(length);
    }

    fn visit_expr_for_loop(&mut self, expression: &'ast syn::ExprForLoop) {
        self.visit_expr(&expression.expr);
        let length = self.bindings.len();
        self.bind_pattern(&expression.pat, None, false);
        self.visit_block(&expression.body);
        self.bindings.truncate(length);
    }

    fn visit_expr_if(&mut self, expression: &'ast syn::ExprIf) {
        let length = self.bindings.len();
        self.visit_condition(&expression.cond);
        self.visit_block(&expression.then_branch);
        self.bindings.truncate(length);
        if let Some((_, otherwise)) = &expression.else_branch {
            self.visit_expr(otherwise);
        }
    }

    fn visit_expr_while(&mut self, expression: &'ast syn::ExprWhile) {
        let length = self.bindings.len();
        self.visit_condition(&expression.cond);
        self.visit_block(&expression.body);
        self.bindings.truncate(length);
    }

    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        let length = self.bindings.len();
        self.bind_pattern(&arm.pat, None, false);
        self.visit_expr(&arm.body);
        self.bindings.truncate(length);
    }
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        let bindings = std::mem::take(&mut self.bindings);
        self.check_parameters(&item.sig);
        self.bind_signature(&item.sig);
        syn::visit::visit_item_fn(self, item);
        self.bindings = bindings;
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let outer = self.in_trait_impl;
        self.in_trait_impl = item.trait_.is_some();
        syn::visit::visit_item_impl(self, item);
        self.in_trait_impl = outer;
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        let bindings = std::mem::take(&mut self.bindings);
        if !self.in_trait_impl {
            self.check_parameters(&item.sig);
        }
        self.bind_signature(&item.sig);
        syn::visit::visit_impl_item_fn(self, item);
        self.bindings = bindings;
    }
    fn visit_block(&mut self, block: &'ast syn::Block) {
        let length = self.bindings.len();
        syn::visit::visit_block(self, block);
        self.bindings.truncate(length);
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let Some(init) = &local.init {
            self.visit_expr(&init.expr);
            if let Some((_, diverge)) = &init.diverge {
                self.visit_expr(diverge);
            }
        }
        let instant = local
            .init
            .as_ref()
            .is_some_and(|init| self.local_instant(&init.expr));
        self.bind_pattern(&local.pat, None, instant);
        self.mark_channel_sender(local);
        self.check_literal_local(local);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        let elapsed_operation = matches!(
            call.method.to_string().as_str(),
            "duration_since" | "checked_duration_since" | "saturating_duration_since"
        ) && self.local_instant(&call.receiver);
        let send = matches!(
            call.method.to_string().as_str(),
            "send" | "try_send" | "send_replace"
        ) && matches!(&*call.receiver, syn::Expr::Path(path) if self.bindings.iter().rev().find(|binding| path.path.is_ident(&binding.name)).is_some_and(|binding| binding.is_sender));
        if !elapsed_operation
            && send
            && call
                .args
                .iter()
                .any(|argument| self.local_instant(argument))
        {
            self.report_used_instant(call.span());
        }
        syn::visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_struct(&mut self, expression: &'ast syn::ExprStruct) {
        for field in &expression.fields {
            if self.local_instant(&field.expr) {
                self.report_used_instant(field.span());
            }
        }
        syn::visit::visit_expr_struct(self, expression);
    }

    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        let logging = self
            .context
            .resolve_external_call_paths(self.ordinal, self.module, &item.path)
            .iter()
            .any(|path| matches!(path.package.as_str(), "tracing" | "log"));
        if (logging || self.is_formatting_macro(&item.path))
            && self.logs_instant(item.tokens.clone())
        {
            self.report_used_instant(item.span());
        }
    }
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
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
    }

    fn visit_field(&mut self, field: &'ast syn::Field) {
        let is_unit_named = !self.is_wire_boundary()
            && !self.in_serde_entry_type
            && field
                .ident
                .as_ref()
                .is_some_and(|ident| self.is_unit_named_primitive(ident, &field.ty));
        if self.instant(&field.ty) {
            self.report_instant(field.span());
        } else if is_unit_named {
            self.report_unit_suffix(field.span());
        }
        syn::visit::visit_field(self, field);
    }

    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        let outer = self.in_serde_entry_type;
        self.in_serde_entry_type = self.is_serde_entry_type(&item.attrs);
        syn::visit::visit_item_struct(self, item);
        self.in_serde_entry_type = outer;
    }

    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        let outer = self.in_serde_entry_type;
        self.in_serde_entry_type = self.is_serde_entry_type(&item.attrs);
        syn::visit::visit_item_enum(self, item);
        self.in_serde_entry_type = outer;
    }

    fn visit_return_type(&mut self, returned: &'ast syn::ReturnType) {
        if let syn::ReturnType::Type(_, typed) = returned
            && self.instant(typed)
        {
            self.report_instant(returned.span());
        }
        syn::visit::visit_return_type(self, returned);
    }

    fn visit_item_const(&mut self, item: &'ast syn::ItemConst) {
        if self.is_unit_named_primitive(&item.ident, &item.ty) {
            self.report_unit_suffix(item.span());
        }
        syn::visit::visit_item_const(self, item);
    }

    fn visit_item_static(&mut self, item: &'ast syn::ItemStatic) {
        if self.is_unit_named_primitive(&item.ident, &item.ty) {
            self.report_unit_suffix(item.span());
        }
        syn::visit::visit_item_static(self, item);
    }

    fn visit_impl_item_const(&mut self, item: &'ast syn::ImplItemConst) {
        if self.is_unit_named_primitive(&item.ident, &item.ty) {
            self.report_unit_suffix(item.span());
        }
        syn::visit::visit_impl_item_const(self, item);
    }

    fn visit_trait_item_const(&mut self, item: &'ast syn::TraitItemConst) {
        if self.is_unit_named_primitive(&item.ident, &item.ty) {
            self.report_unit_suffix(item.span());
        }
        syn::visit::visit_trait_item_const(self, item);
    }
}

struct TimeViolation {
    span: proc_macro2::Span,
    message: &'static str,
}

struct LocalBinding {
    name: String,
    is_instant: bool,
    is_sender: bool,
}

impl Rule for TimePrimitive {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
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
                let mut visitor = TimeVisitor {
                    context,
                    ordinal,
                    module: module.id,
                    violations: Vec::new(),
                    bindings: Vec::new(),
                    in_serde_entry_type: false,
                    in_trait_impl: false,
                };
                visitor.visit_file(&file.syntax);
                for violation in visitor.violations {
                    findings.push(build_node_finding(
                        "TimePrimitive",
                        &file.path,
                        find_node_span(violation.span),
                        format_module_path(&index.crate_name, &file.module.path),
                        violation.message.to_owned(),
                    ));
                }
            }
        }
        findings
    }
}
