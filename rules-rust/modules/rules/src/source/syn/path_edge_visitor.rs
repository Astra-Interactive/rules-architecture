use crate::source::model::CfgPredicate;
use crate::source::model::CrateIndex;
use crate::source::model::NodeSpan;
use crate::source::model::Resolution;
use crate::source::model::SourceEdge;
use crate::source::model::SourceReference;
use crate::source::model::SourceTree;
use crate::source::model::UnanalysableSite;
use crate::source::model::UnanalysableSource;
use crate::source::util::find_node_span;
use crate::source::util::parse_macro_arguments;
use std::collections::BTreeSet;
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct PathEdgeVisitor<'a> {
    index: &'a CrateIndex,
    module: usize,
    file: String,
    anchor: NodeSpan,
    cfg: CfgPredicate,
    scopes: Vec<BTreeSet<String>>,
    type_parameters: BTreeSet<String>,
    is_type_path: bool,
    pub(crate) edges: Vec<SourceEdge>,
    pub(crate) unanalysable: Vec<UnanalysableSource>,
    pub(crate) references: Vec<SourceReference>,
}

impl<'a> PathEdgeVisitor<'a> {
    fn bind_block_item(&mut self, item: &syn::Item) {
        let name = match item {
            syn::Item::Const(item) => Some(item.ident.unraw().to_string()),
            syn::Item::Static(item) => Some(item.ident.unraw().to_string()),
            syn::Item::Fn(item) => Some(item.sig.ident.unraw().to_string()),
            syn::Item::Struct(item) => Some(item.ident.unraw().to_string()),
            syn::Item::Enum(item) => Some(item.ident.unraw().to_string()),
            syn::Item::Union(item) => Some(item.ident.unraw().to_string()),
            syn::Item::Type(item) => Some(item.ident.unraw().to_string()),
            syn::Item::Trait(item) => Some(item.ident.unraw().to_string()),
            syn::Item::Mod(item) => Some(item.ident.unraw().to_string()),
            _ => None,
        };
        if let Some(name) = name
            && let Some(scope) = self.scopes.last_mut()
        {
            scope.insert(name.clone());
            if matches!(
                item,
                syn::Item::Struct(_)
                    | syn::Item::Enum(_)
                    | syn::Item::Union(_)
                    | syn::Item::Type(_)
                    | syn::Item::Trait(_)
                    | syn::Item::Mod(_)
            ) {
                self.type_parameters.insert(name);
            }
        }
    }

    fn visit_local_import(&mut self, tree: &syn::UseTree, prefix: Vec<String>) {
        match tree {
            syn::UseTree::Path(path) => {
                let mut prefix = prefix;
                prefix.push(path.ident.unraw().to_string());
                self.visit_local_import(&path.tree, prefix);
            }
            syn::UseTree::Group(group) => {
                for child in &group.items {
                    self.visit_local_import(child, prefix.clone());
                }
            }
            other => {
                let mut prefix = prefix;
                let name = match other {
                    syn::UseTree::Name(name) => {
                        prefix.push(name.ident.unraw().to_string());
                        Some(name.ident.unraw().to_string())
                    }
                    syn::UseTree::Rename(rename) => {
                        prefix.push(rename.ident.unraw().to_string());
                        Some(rename.rename.unraw().to_string())
                    }
                    _ => None,
                };
                if let Ok(path) = syn::parse_str::<syn::Path>(&prefix.join("::")) {
                    let start = self.edges.len();
                    self.record_resolution(&path);
                    for edge in &mut self.edges[start..] {
                        edge.span = find_node_span(tree.span());
                    }
                }
                if let (Some(scope), Some(name)) = (self.scopes.last_mut(), name) {
                    self.type_parameters.insert(name.clone());
                    scope.insert(name);
                }
            }
        }
    }

    pub(crate) fn new(
        index: &'a CrateIndex,
        module: usize,
        span: NodeSpan,
        cfg: CfgPredicate,
    ) -> Self {
        Self {
            index,
            module,
            file: index.modules[module].file.clone(),
            anchor: span,
            cfg,
            scopes: vec![BTreeSet::new()],
            type_parameters: BTreeSet::new(),
            is_type_path: false,
            edges: Vec::new(),
            unanalysable: Vec::new(),
            references: Vec::new(),
        }
    }

    fn bind_pattern(&mut self, pattern: &syn::Pat) {
        match pattern {
            syn::Pat::Ident(binding) => {
                if let Some(scope) = self.scopes.last_mut() {
                    scope.insert(binding.ident.unraw().to_string());
                }
                if let Some((_, nested)) = &binding.subpat {
                    self.bind_pattern(nested);
                }
            }
            syn::Pat::Tuple(tuple) => {
                for part in &tuple.elems {
                    self.bind_pattern(part);
                }
            }
            syn::Pat::TupleStruct(tuple) => {
                for part in &tuple.elems {
                    self.bind_pattern(part);
                }
            }
            syn::Pat::Struct(record) => {
                for field in &record.fields {
                    self.bind_pattern(&field.pat);
                }
            }
            syn::Pat::Reference(reference) => self.bind_pattern(&reference.pat),
            syn::Pat::Type(typed) => self.bind_pattern(&typed.pat),
            syn::Pat::Slice(slice) => {
                for part in &slice.elems {
                    self.bind_pattern(part);
                }
            }
            syn::Pat::Or(pattern) => {
                for part in &pattern.cases {
                    self.bind_pattern(part);
                }
            }
            syn::Pat::Paren(paren) => self.bind_pattern(&paren.pat),
            syn::Pat::Guard(guard) => self.bind_pattern(&guard.pat),
            _ => {}
        }
    }

    fn is_logging_macro(&self, path: &syn::Path) -> bool {
        let Resolution::External(path) = self.index.resolve_path(self.module, path) else {
            return false;
        };
        let mut segments = path.split("::");
        let Some(root) = segments.next() else {
            return false;
        };
        let package = self
            .index
            .dependency_packages
            .get(root)
            .map_or(root, String::as_str);
        matches!(package, "tracing" | "log")
            && matches!(
                segments.next(),
                Some("trace" | "debug" | "info" | "warn" | "error" | "event" | "span")
            )
            && segments.next().is_none()
    }

    pub(crate) fn record_resolution(&mut self, path: &syn::Path) {
        let Some(first) = path.segments.first() else {
            return;
        };
        let name = first.ident.unraw().to_string();
        if name == "self" && path.segments.len() == 1 {
            return;
        }
        if (self.is_type_path && self.type_parameters.contains(&name))
            || (!self.is_type_path && self.scopes.iter().rev().any(|scope| scope.contains(&name)))
        {
            return;
        }
        let resolution = self.index.resolve_path(self.module, path);
        if matches!(
            resolution,
            Resolution::Internal(_) | Resolution::External(_)
        ) {
            self.references.push(SourceReference {
                module: self.module,
                file: self.file.clone(),
                span: find_node_span(path.span()),
                anchor: self.anchor,
                resolution: resolution.clone(),
                cfg: self.cfg.clone(),
            });
        }
        match resolution {
            Resolution::Internal(targets) => {
                for target in targets {
                    self.edges.push(SourceEdge {
                        source: self.module,
                        target: target.module,
                        item: target.item,
                        file: self.file.clone(),
                        span: find_node_span(path.span()),
                        cfg: self.cfg.and(&target.cfg),
                        is_test: self.index.modules[self.module].tree != SourceTree::Src,
                        path: path
                            .segments
                            .iter()
                            .map(|segment| segment.ident.unraw().to_string())
                            .collect(),
                    });
                }
            }
            Resolution::Unresolved => {
                let subject = crate::source::util::format_tokens(path);
                if !self.unanalysable.iter().any(|existing| existing.subject == subject && matches!(&existing.site, UnanalysableSite::Node {span,..} if *span == self.anchor)) {
                    self.unanalysable.push(UnanalysableSource { site: UnanalysableSite::Node { file: self.file.clone(), span: self.anchor }, subject: subject.clone(), message: format!("the syntactic resolver cannot resolve `{subject}`") });
                }
            }
            _ => {}
        }
    }
}

impl<'ast> Visit<'ast> for PathEdgeVisitor<'_> {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        if matches!(item, syn::Item::Use(_)) {
            syn::visit::visit_item(self, item);
            return;
        }
        let parameters = self.type_parameters.clone();
        self.scopes.push(BTreeSet::new());
        syn::visit::visit_item(self, item);
        self.scopes.pop();
        self.type_parameters = parameters;
    }
    fn visit_field(&mut self, field: &'ast syn::Field) {
        let previous = self.cfg.clone();
        self.cfg = self.cfg.and(&CfgPredicate::from_attributes(&field.attrs));
        syn::visit::visit_field(self, field);
        self.cfg = previous;
    }

    fn visit_variant(&mut self, variant: &'ast syn::Variant) {
        let previous = self.cfg.clone();
        self.cfg = self.cfg.and(&CfgPredicate::from_attributes(&variant.attrs));
        syn::visit::visit_variant(self, variant);
        self.cfg = previous;
    }

    fn visit_expr(&mut self, expression: &'ast syn::Expr) {
        let previous = self.cfg.clone();
        self.cfg = self.cfg.and(&CfgPredicate::from_attributes(
            crate::source::util::find_expression_attributes(expression),
        ));
        syn::visit::visit_expr(self, expression);
        self.cfg = previous;
    }

    fn visit_attribute(&mut self, _attribute: &'ast syn::Attribute) {}

    fn visit_path(&mut self, path: &'ast syn::Path) {
        self.record_resolution(path);
        syn::visit::visit_path(self, path);
    }

    fn visit_expr_path(&mut self, expression: &'ast syn::ExprPath) {
        if expression
            .qself
            .as_ref()
            .is_some_and(|qualified| qualified.position == 0)
        {
            if let Some(qualified) = &expression.qself {
                self.visit_type(&qualified.ty);
            }
            for segment in &expression.path.segments {
                self.visit_path_arguments(&segment.arguments);
            }
            return;
        }
        let previous = self.is_type_path;
        self.is_type_path = false;
        syn::visit::visit_expr_path(self, expression);
        self.is_type_path = previous;
    }

    fn visit_type_path(&mut self, typed: &'ast syn::TypePath) {
        if typed
            .qself
            .as_ref()
            .is_some_and(|qualified| qualified.position == 0)
        {
            if let Some(qualified) = &typed.qself {
                self.visit_type(&qualified.ty);
            }
            for segment in &typed.path.segments {
                self.visit_path_arguments(&segment.arguments);
            }
            return;
        }
        let previous = self.is_type_path;
        self.is_type_path = true;
        syn::visit::visit_type_path(self, typed);
        self.is_type_path = previous;
    }

    fn visit_generics(&mut self, generics: &'ast syn::Generics) {
        for parameter in &generics.params {
            let name = match parameter {
                syn::GenericParam::Type(parameter) => Some(parameter.ident.unraw().to_string()),
                syn::GenericParam::Const(parameter) => Some(parameter.ident.unraw().to_string()),
                _ => None,
            };
            if let (Some(scope), Some(name)) = (self.scopes.last_mut(), name) {
                self.type_parameters.insert(name.clone());
                scope.insert(name);
            }
        }
        syn::visit::visit_generics(self, generics);
    }

    fn visit_signature(&mut self, signature: &'ast syn::Signature) {
        for input in &signature.inputs {
            if let syn::FnArg::Typed(argument) = input {
                self.bind_pattern(&argument.pat);
            }
        }
        syn::visit::visit_signature(self, signature);
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        let parameters = self.type_parameters.clone();
        self.scopes.push(BTreeSet::new());
        for statement in &block.stmts {
            if let syn::Stmt::Item(item) = statement {
                self.bind_block_item(item);
            }
        }
        syn::visit::visit_block(self, block);
        self.scopes.pop();
        self.type_parameters = parameters;
    }

    fn visit_stmt(&mut self, statement: &'ast syn::Stmt) {
        let previous = self.anchor;
        self.anchor = find_node_span(statement.span());
        syn::visit::visit_stmt(self, statement);
        self.anchor = previous;
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let Some(initializer) = &local.init {
            self.visit_local_init(initializer);
        }
        self.visit_pat(&local.pat);
        self.bind_pattern(&local.pat);
    }

    fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure) {
        self.scopes.push(BTreeSet::new());
        for argument in &closure.inputs {
            self.bind_pattern(argument);
        }
        syn::visit::visit_expr_closure(self, closure);
        self.scopes.pop();
    }

    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        self.scopes.push(BTreeSet::new());
        self.bind_pattern(&arm.pat);
        syn::visit::visit_arm(self, arm);
        self.scopes.pop();
    }

    fn visit_expr_for_loop(&mut self, expression: &'ast syn::ExprForLoop) {
        self.visit_expr(&expression.expr);
        self.scopes.push(BTreeSet::new());
        self.bind_pattern(&expression.pat);
        self.visit_pat(&expression.pat);
        self.visit_block(&expression.body);
        self.scopes.pop();
    }

    fn visit_macro(&mut self, expression: &'ast syn::Macro) {
        if expression.path.is_ident("matches") {
            let mut tokens = expression.tokens.clone().into_iter();
            let mut input = proc_macro2::TokenStream::new();
            for token in tokens.by_ref() {
                if matches!(&token, proc_macro2::TokenTree::Punct(punctuation) if punctuation.as_char() == ',')
                {
                    break;
                }
                input.extend([token]);
            }
            let mut pattern: Vec<_> = tokens.collect();
            if matches!(pattern.last(), Some(proc_macro2::TokenTree::Punct(punctuation)) if punctuation.as_char() == ',')
            {
                pattern.pop();
            }
            let pattern: proc_macro2::TokenStream = pattern.into_iter().collect();
            if let Ok(expression) =
                syn::parse2::<syn::ExprMatch>(quote::quote!(match #input { #pattern => () }))
            {
                self.visit_expr_match(&expression);
            }
            return;
        }
        let logging = self.is_logging_macro(&expression.path);
        for argument in &parse_macro_arguments(expression) {
            if logging && let syn::Expr::Assign(field) = argument {
                self.visit_expr(&field.right);
            } else {
                self.visit_expr(argument);
            }
        }
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        let parameters = self.type_parameters.clone();
        self.scopes.push(BTreeSet::new());
        syn::visit::visit_item_fn(self, item);
        self.scopes.pop();
        self.type_parameters = parameters;
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        let parameters = self.type_parameters.clone();
        let predicate = self.cfg.clone();
        self.cfg = self.cfg.and(&CfgPredicate::from_attributes(&item.attrs));
        self.scopes.push(BTreeSet::new());
        syn::visit::visit_impl_item_fn(self, item);
        self.scopes.pop();
        self.type_parameters = parameters;
        self.cfg = predicate;
    }

    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        let parameters = self.type_parameters.clone();
        let predicate = self.cfg.clone();
        self.cfg = self.cfg.and(&CfgPredicate::from_attributes(&item.attrs));
        self.scopes.push(BTreeSet::new());
        syn::visit::visit_trait_item_fn(self, item);
        self.scopes.pop();
        self.type_parameters = parameters;
        self.cfg = predicate;
    }

    fn visit_expr_if(&mut self, expression: &'ast syn::ExprIf) {
        self.scopes.push(BTreeSet::new());
        self.visit_expr(&expression.cond);
        self.visit_block(&expression.then_branch);
        self.scopes.pop();
        if let Some((_, otherwise)) = &expression.else_branch {
            self.visit_expr(otherwise);
        }
    }

    fn visit_expr_while(&mut self, expression: &'ast syn::ExprWhile) {
        self.scopes.push(BTreeSet::new());
        syn::visit::visit_expr_while(self, expression);
        self.scopes.pop();
    }

    fn visit_expr_let(&mut self, expression: &'ast syn::ExprLet) {
        self.visit_expr(&expression.expr);
        self.visit_pat(&expression.pat);
        self.bind_pattern(&expression.pat);
    }

    fn visit_item_mod(&mut self, _module: &'ast syn::ItemMod) {}
    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        self.visit_local_import(&item.tree, Vec::new());
    }
}
