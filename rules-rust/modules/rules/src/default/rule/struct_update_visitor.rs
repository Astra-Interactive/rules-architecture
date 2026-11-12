use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::check::util::format_module_path;
use crate::default::rule::StructDefaultValue;
use crate::module::util::owns_implementation;
use crate::source::model::CfgPredicate;
use crate::source::model::CrateIndex;
use crate::source::model::IndexedItem;
use crate::source::model::Resolution;
use crate::source::model::SourceFile;
use crate::source::util::find_node_span;
use crate::source::util::parse_macro_expressions;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct StructUpdateVisitor<'source> {
    index: &'source CrateIndex,
    file: &'source SourceFile,
    module: usize,
    self_type: Option<syn::Path>,
    cfg: CfgPredicate,
    findings: Vec<RuleFinding>,
}

impl StructUpdateVisitor<'_> {
    fn is_default_exempt(&self, identity: usize, item: &IndexedItem) -> bool {
        let attributes = match &item.syntax {
            syn::Item::Struct(value) => &value.attrs,
            syn::Item::Enum(value) => &value.attrs,
            _ => return false,
        };
        let has_empty_new = self.index.implementations.iter().any(|implementation| {
            implementation.syntax.trait_.is_none()
                && owns_implementation(self.index, identity, implementation)
                && implementation.syntax.items.iter().any(|member| {
                    matches!(member, syn::ImplItem::Fn(function)
                        if function.sig.ident == "new" && function.sig.inputs.is_empty())
                })
        });
        StructDefaultValue::derives(attributes, "Deserialize") || has_empty_new
    }

    fn is_own_data(&self, module: usize, path: &syn::Path, depth: usize) -> bool {
        if depth > 8 {
            return false;
        }
        let Resolution::Internal(targets) = self.index.resolve_path(module, path) else {
            return false;
        };
        targets.iter().any(|target| {
            let Some(identity) = target.item else { return false; };
            let Some(item) = self.index.items.get(identity) else { return false; };
            if !item.cfg.and(&self.cfg).is_satisfiable() { return false; }
            match &item.syntax {
                syn::Item::Struct(_) | syn::Item::Enum(_) => !self.is_default_exempt(identity, item),
                syn::Item::Type(alias) => matches!(&*alias.ty, syn::Type::Path(path) if self.is_own_data(item.module, &path.path, depth + 1)),
                _ => false,
            }
        })
    }

    fn is_default_call(expression: &syn::Expr) -> bool {
        let syn::Expr::Call(call) = expression else {
            return false;
        };
        let syn::Expr::Path(path) = &*call.func else {
            return false;
        };
        call.args.is_empty()
            && path.path.segments.len() >= 2
            && path
                .path
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "default")
    }

    pub(crate) fn check_context(context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for file in context.crates.iter().flat_map(|sources| &sources.files) {
            let Some(index) = context.indexes.iter().find(|index| {
                index.crate_name == file.module.crate_name && index.target == file.module.target
            }) else {
                continue;
            };
            let Some(module) = index
                .modules
                .iter()
                .find(|module| module.module == file.module)
            else {
                continue;
            };
            let mut visitor = StructUpdateVisitor {
                index,
                file,
                module: module.id,
                self_type: None,
                cfg: module.cfg.clone(),
                findings: Vec::new(),
            };
            visitor.visit_file(&file.syntax);
            findings.extend(visitor.findings);
        }
        findings
    }
}

impl<'ast> Visit<'ast> for StructUpdateVisitor<'_> {
    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        for argument in &parse_macro_expressions(item) {
            self.visit_expr(argument);
        }
    }

    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        let parent = self.module;
        let parent_cfg = self.cfg.clone();
        if let Some(module) = self.index.modules.iter().find(|module| {
            module.parent == Some(parent)
                && module.span == find_node_span(item.span())
                && module
                    .module
                    .path
                    .last()
                    .is_some_and(|name| item.ident == name)
        }) {
            self.module = module.id;
            self.cfg = module.cfg.clone();
        }
        syn::visit::visit_item_mod(self, item);
        self.module = parent;
        self.cfg = parent_cfg;
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let parent = self.self_type.take();
        let parent_cfg = self.cfg.clone();
        self.cfg = self.cfg.and(&CfgPredicate::from_attributes(&item.attrs));
        self.self_type = match &*item.self_ty {
            syn::Type::Path(path) => Some(path.path.clone()),
            _ => None,
        };
        syn::visit::visit_item_impl(self, item);
        self.self_type = parent;
        self.cfg = parent_cfg;
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        let parent = self.cfg.clone();
        self.cfg = self.cfg.and(&CfgPredicate::from_attributes(&item.attrs));
        syn::visit::visit_item_fn(self, item);
        self.cfg = parent;
    }

    fn visit_expr_struct(&mut self, expression: &'ast syn::ExprStruct) {
        if expression
            .rest
            .as_ref()
            .is_some_and(|base| Self::is_default_call(base))
        {
            let path = if expression.path.is_ident("Self") {
                self.self_type.as_ref().unwrap_or(&expression.path)
            } else {
                &expression.path
            };
            if self.is_own_data(self.module, path, 0) {
                self.findings.push(build_node_finding("StructDefaultValue", &self.file.path, find_node_span(expression.span()), format_module_path(&self.index.crate_name, &self.index.modules[self.module].module.path), "Pass every own-type field explicitly instead of filling it with default() (struct-default-values.rs.md)".to_owned()));
            }
        }
        syn::visit::visit_expr_struct(self, expression);
    }
}
