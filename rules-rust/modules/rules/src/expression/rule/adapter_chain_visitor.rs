use crate::source::model::NodeSpan;
use crate::source::util::{find_node_span, parse_macro_arguments};
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct AdapterChainVisitor<'context> {
    context: &'context crate::check::model::RuleContext,
    ordinal: usize,
    module: usize,
    file: String,
    strings: std::collections::HashSet<String>,
    bindings: std::collections::HashSet<String>,
    parent_method: Option<String>,
    spans: Vec<NodeSpan>,
}

impl<'context> AdapterChainVisitor<'context> {
    pub(crate) fn collect(
        context: &'context crate::check::model::RuleContext,
        ordinal: usize,
        file: &crate::source::model::SourceFile,
    ) -> Vec<NodeSpan> {
        let module = context.indexes[ordinal]
            .modules
            .iter()
            .find(|module| module.module == file.module)
            .map_or(0, |module| module.id);
        let mut visitor = Self {
            context,
            ordinal,
            module,
            file: file.path.clone(),
            strings: std::collections::HashSet::new(),
            bindings: std::collections::HashSet::new(),
            parent_method: None,
            spans: Vec::new(),
        };
        visitor.visit_file(&file.syntax);
        visitor.spans
    }

    fn string_reference(&self, ordinal: usize, module: usize, typed: &syn::Type) -> bool {
        self.context.resolve_type_syntax(ordinal, module, typed).iter().any(|resolved| {
            matches!(&resolved.typed, syn::Type::Reference(reference)
                if reference.mutability.is_none() && matches!(reference.elem.as_ref(), syn::Type::Path(path)
                    if path.path.is_ident("str") && matches!(self.context.indexes[resolved.index].resolve_path(resolved.module, &path.path), crate::source::model::Resolution::Local)))
        })
    }

    fn is_literal_pattern(argument: &syn::Expr) -> bool {
        matches!(argument, syn::Expr::Lit(literal)
            if matches!(literal.lit, syn::Lit::Char(_) | syn::Lit::Str(_)))
    }

    fn string_lines(&self, expression: &syn::Expr) -> bool {
        let syn::Expr::MethodCall(lines) = expression else {
            return false;
        };
        let name = lines.method.to_string();
        let lines_iterator =
            Self::WHOLE_STRING_ITERATORS.contains(&name.as_str()) && lines.args.is_empty();
        let split_iterator = Self::PATTERN_STRING_ITERATORS.contains(&name.as_str())
            && lines.args.len() == 1
            && lines.args.first().is_some_and(Self::is_literal_pattern);
        if !lines_iterator && !split_iterator {
            return false;
        }
        match lines.receiver.as_ref() {
            syn::Expr::Path(path) => path
                .path
                .get_ident()
                .is_some_and(|name| self.strings.contains(&name.to_string())),
            syn::Expr::Lit(literal) => matches!(literal.lit, syn::Lit::Str(_)),
            _ => false,
        }
    }

    fn target_feature(meta: &syn::Meta) -> bool {
        if meta.path().is_ident("target_feature") {
            return true;
        }
        let syn::Meta::List(list) = meta else {
            return false;
        };
        if !list.path.is_ident("cfg_attr") {
            return false;
        }
        list.parse_args_with(
            syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
        )
        .is_ok_and(|entries| entries.iter().skip(1).any(Self::target_feature))
    }

    fn safe_function_path(function: &syn::ItemFn) -> bool {
        let signature = &function.sig;
        let constrained_generic = signature
            .generics
            .params
            .iter()
            .any(|parameter| !matches!(parameter, syn::GenericParam::Lifetime(_)));
        !matches!(signature.safety, syn::Safety::Unsafe(_))
            && !constrained_generic
            && !function
                .attrs
                .iter()
                .any(|attribute| Self::target_feature(&attribute.meta))
    }

    fn is_owned_string(&self, ordinal: usize, module: usize, typed: &syn::Type) -> bool {
        let candidates = self.context.resolve_type_syntax(ordinal, module, typed);
        candidates.iter().any(|candidate| {
            let syn::Type::Path(path) = &candidate.typed else {
                return false;
            };
            let identity = self.context.canonical_external_type_path(
                candidate.index,
                candidate.module,
                &path.path,
            );
            identity.is_some_and(|external| {
                ["std", "alloc"].contains(&external.package.as_str())
                    && external
                        .segments
                        .iter()
                        .skip(1)
                        .map(String::as_str)
                        .eq(["string", "String"])
            })
        })
    }

    fn borrows_result(method: &syn::Ident) -> Option<bool> {
        let is_trimming = ["trim", "trim_start", "trim_end"]
            .iter()
            .any(|name| method == *name);
        if is_trimming {
            return Some(true);
        }
        [
            "to_ascii_lowercase",
            "to_ascii_uppercase",
            "to_lowercase",
            "to_uppercase",
            "to_owned",
            "to_string",
        ]
        .iter()
        .any(|name| method == *name)
        .then_some(false)
    }

    fn accepts_adaptation(
        &self,
        borrowed: bool,
        ordinal: usize,
        module: usize,
        typed: &syn::Type,
    ) -> bool {
        if borrowed {
            self.string_reference(ordinal, module, typed)
        } else {
            self.is_owned_string(ordinal, module, typed)
        }
    }

    fn two_step(&self, call: &syn::ExprMethodCall) -> bool {
        if call.method != "map" || call.args.len() != 1 || !self.string_lines(&call.receiver) {
            return false;
        }
        let Some(syn::Expr::Closure(closure)) = call.args.first() else {
            return false;
        };
        if closure.inputs.len() != 1 || closure.asyncness.is_some() {
            return false;
        }
        let Some(syn::Pat::Ident(parameter)) = closure.inputs.first() else {
            return false;
        };
        let syn::Expr::Call(function) = closure.body.as_ref() else {
            return false;
        };
        let syn::Expr::Path(path) = function.func.as_ref() else {
            return false;
        };
        if function.args.len() != 1
            || path.qself.is_some()
            || path
                .path
                .get_ident()
                .is_some_and(|name| self.bindings.contains(&name.to_string()))
        {
            return false;
        }
        let Some(syn::Expr::MethodCall(adaptation)) = function.args.first() else {
            return false;
        };
        let Some(borrowed) = Self::borrows_result(&adaptation.method) else {
            return false;
        };
        if !adaptation.args.is_empty() {
            return false;
        }
        if !matches!(adaptation.receiver.as_ref(), syn::Expr::Path(path) if path.path.is_ident(&parameter.ident.to_string()))
        {
            return false;
        }
        let targets = self
            .context
            .resolve_workspace_path(self.ordinal, self.module, &path.path);
        !targets.is_empty() && targets.iter().all(|target| target.path.item.is_some_and(|identity| {
            let item = &self.context.indexes[target.index].items[identity];
            let syn::Item::Fn(function) = &item.syntax else { return false; };
            Self::safe_function_path(function) && function.sig.inputs.len() == 1 && matches!(function.sig.inputs.first(), Some(syn::FnArg::Typed(input)) if self.accepts_adaptation(borrowed, target.index, item.module, &input.ty))
        }))
    }

    fn adapter(call: &syn::ExprMethodCall) -> bool {
        [
            "map",
            "map_err",
            "and_then",
            "filter",
            "filter_map",
            "flat_map",
            "inspect",
            "inspect_err",
            "or_else",
            "map_while",
            "skip_while",
            "take_while",
        ]
        .iter()
        .any(|name| call.method == *name)
            && call.args.len() == 1
            && matches!(call.args.first(), Some(syn::Expr::Path(_)))
    }

    fn length(expression: &syn::Expr, method: &syn::Ident) -> usize {
        match expression {
            syn::Expr::MethodCall(call) if call.method == *method && Self::adapter(call) => {
                1 + Self::length(&call.receiver, method)
            }
            syn::Expr::Paren(paren) => Self::length(&paren.expr, method),
            syn::Expr::Group(group) => Self::length(&group.expr, method),
            _ => 0,
        }
    }

    fn visit_function(&mut self, signature: &syn::Signature, block: &syn::Block) {
        let previous = std::mem::take(&mut self.strings);
        let bindings = std::mem::take(&mut self.bindings);
        for input in &signature.inputs {
            let syn::FnArg::Typed(input) = input else {
                continue;
            };
            let syn::Pat::Ident(binding) = input.pat.as_ref() else {
                continue;
            };
            self.bindings.insert(binding.ident.to_string());
            if self.string_reference(self.ordinal, self.module, &input.ty) {
                self.strings.insert(binding.ident.to_string());
            }
        }
        self.visit_block(block);
        self.strings = previous;
        self.bindings = bindings;
    }

    const WHOLE_STRING_ITERATORS: [&'static str; 3] =
        ["lines", "split_whitespace", "split_ascii_whitespace"];
    const PATTERN_STRING_ITERATORS: [&'static str; 4] =
        ["split", "rsplit", "split_terminator", "rsplit_terminator"];
}

impl<'ast> Visit<'ast> for AdapterChainVisitor<'_> {
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        let previous = self.module;
        let span = find_node_span(item.span());
        if let Some(module) = self.context.indexes[self.ordinal]
            .modules
            .iter()
            .find(|module| module.file == self.file && module.span == span)
        {
            self.module = module.id;
        }
        syn::visit::visit_item_mod(self, item);
        self.module = previous;
    }

    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        self.visit_function(&function.sig, &function.block);
    }

    fn visit_impl_item_fn(&mut self, function: &'ast syn::ImplItemFn) {
        self.visit_function(&function.sig, &function.block);
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        let previous = self.strings.clone();
        let bindings = self.bindings.clone();
        syn::visit::visit_block(self, block);
        self.strings = previous;
        self.bindings = bindings;
    }

    fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure) {
        let strings = self.strings.clone();
        let bindings = self.bindings.clone();
        syn::visit::visit_expr_closure(self, closure);
        self.strings = strings;
        self.bindings = bindings;
    }

    fn visit_pat_ident(&mut self, pattern: &'ast syn::PatIdent) {
        self.strings.remove(&pattern.ident.to_string());
        self.bindings.insert(pattern.ident.to_string());
        syn::visit::visit_pat_ident(self, pattern);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        let adapter = Self::adapter(call);
        let continues_run = self
            .parent_method
            .as_ref()
            .is_some_and(|name| call.method == *name);
        if adapter && !continues_run && Self::length(&call.receiver, &call.method) >= 2 {
            self.spans.push(find_node_span(call.span()));
        }
        if self.two_step(call) {
            self.spans.push(find_node_span(call.span()));
        }
        let parent = self.parent_method.take();
        self.parent_method = adapter.then(|| call.method.to_string());
        self.visit_expr(&call.receiver);
        self.parent_method = None;
        for argument in &call.args {
            self.visit_expr(argument);
        }
        self.parent_method = parent;
    }

    fn visit_macro(&mut self, expression: &'ast syn::Macro) {
        for argument in &parse_macro_arguments(expression) {
            self.visit_expr(argument);
        }
    }
}
