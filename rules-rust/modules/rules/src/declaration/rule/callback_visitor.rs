use crate::declaration::util::has_bound;
use crate::source::model::SourceFile;
use crate::source::util::parse_macro_arguments;
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct CallbackVisitor<'context> {
    context: &'context crate::check::model::RuleContext,
    ordinal: usize,
    module: usize,
    closure_parameters: Vec<String>,
    is_override: bool,
    is_test: bool,
    spans: Vec<proc_macro2::Span>,
}

impl<'context> CallbackVisitor<'context> {
    pub(crate) fn new(
        context: &'context crate::check::model::RuleContext,
        ordinal: usize,
        module: usize,
    ) -> Self {
        Self {
            context,
            ordinal,
            module,
            closure_parameters: Vec::new(),
            is_override: false,
            is_test: false,
            spans: Vec::new(),
        }
    }

    fn has_closure_bound(
        bounds: &syn::punctuated::Punctuated<syn::TypeParamBound, syn::Token![+]>,
    ) -> bool {
        ["Fn", "FnMut", "FnOnce"]
            .iter()
            .any(|name| has_bound(bounds, name))
    }

    fn push_generics(&mut self, generics: &syn::Generics) {
        for parameter in generics.type_params() {
            if Self::has_closure_bound(&parameter.bounds) {
                self.closure_parameters
                    .push(parameter.ident.unraw().to_string());
            }
        }
        if let Some(clause) = &generics.where_clause {
            for predicate in &clause.predicates {
                let syn::WherePredicate::Type(predicate) = predicate else {
                    continue;
                };
                let syn::Type::Path(path) = &predicate.bounded_ty else {
                    continue;
                };
                if path.path.segments.len() == 1 && Self::has_closure_bound(&predicate.bounds) {
                    self.closure_parameters
                        .push(path.path.segments[0].ident.unraw().to_string());
                }
            }
        }
    }

    fn is_closure_type(&self, value: &syn::Type) -> bool {
        match value {
            syn::Type::FnPtr(_) => true,
            syn::Type::ImplTrait(value) => Self::has_closure_bound(&value.bounds),
            syn::Type::TraitObject(value) => Self::has_closure_bound(&value.bounds),
            syn::Type::Reference(value) => self.is_closure_type(&value.elem),
            syn::Type::Ptr(value) => self.is_closure_type(&value.elem),
            syn::Type::Paren(value) => self.is_closure_type(&value.elem),
            syn::Type::Group(value) => self.is_closure_type(&value.elem),
            syn::Type::Path(value) => value.path.segments.iter().any(|segment| {
                self.closure_parameters.iter().any(|name| segment.ident == name)
                    || (["Option", "Box", "Arc", "Rc", "Pin"].iter().any(|name| segment.ident == name)
                        && matches!(&segment.arguments, syn::PathArguments::AngleBracketed(arguments)
                        if arguments.args.iter().any(|argument| matches!(argument, syn::GenericArgument::Type(value) if self.is_closure_type(value))))
                    )
            }),
            _ => false,
        }
    }

    pub(crate) fn collect_spans(mut self, file: &SourceFile) -> Vec<proc_macro2::Span> {
        self.is_test = file
            .module
            .path
            .iter()
            .any(|segment| matches!(segment.as_str(), "test" | "fixture"));
        self.visit_file(&file.syntax);
        self.spans
    }
}

impl<'ast> Visit<'ast> for CallbackVisitor<'_> {
    fn visit_item_enum(&mut self, _item: &'ast syn::ItemEnum) {}

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        let parent = self.is_override;
        self.is_override = false;
        syn::visit::visit_item_fn(self, item);
        self.is_override = parent;
    }
    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        for argument in &parse_macro_arguments(item) {
            self.visit_expr(argument);
        }
    }

    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        let module = self.module;
        if let Some(child) = self.context.indexes[self.ordinal]
            .modules
            .iter()
            .find(|child| {
                child.parent == Some(module)
                    && child.span == crate::source::util::find_node_span(item.span())
            })
        {
            self.module = child.id;
        }
        let parent = self.is_test;
        self.is_test |= item.ident == "test" || item.ident == "fixture";
        let production = crate::source::model::CfgPredicate::Not(Box::new(
            crate::source::model::CfgPredicate::Atom {
                key: "test".to_owned(),
                value: None,
            },
        ));
        self.is_test |= !crate::source::model::CfgPredicate::from_attributes(&item.attrs)
            .and(&production)
            .is_satisfiable();
        syn::visit::visit_item_mod(self, item);
        self.is_test = parent;
        self.module = module;
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let parent = self.is_override;
        let count = self.closure_parameters.len();
        self.is_override = item.trait_.is_some();
        self.push_generics(&item.generics);
        syn::visit::visit_item_impl(self, item);
        self.closure_parameters.truncate(count);
        self.is_override = parent;
    }

    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        let count = self.closure_parameters.len();
        self.push_generics(&item.generics);
        syn::visit::visit_item_struct(self, item);
        self.closure_parameters.truncate(count);
    }

    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        let name = item.ident.unraw().to_string();
        if !self.is_test && (name.ends_with("Listener") || name.ends_with("Callback")) {
            self.spans.push(item.ident.span());
        }
        let count = self.closure_parameters.len();
        self.push_generics(&item.generics);
        syn::visit::visit_item_trait(self, item);
        self.closure_parameters.truncate(count);
    }

    fn visit_signature(&mut self, signature: &'ast syn::Signature) {
        let count = self.closure_parameters.len();
        self.push_generics(&signature.generics);
        if !self.is_override && !self.is_test {
            for argument in &signature.inputs {
                let syn::FnArg::Typed(parameter) = argument else {
                    continue;
                };
                let syn::Pat::Ident(name) = &*parameter.pat else {
                    continue;
                };
                if name.ident.unraw().to_string().starts_with("on_")
                    && (self.is_closure_type(&parameter.ty)
                        || crate::declaration::util::callback_field_type(
                            self.context,
                            self.ordinal,
                            self.module,
                            &parameter.ty,
                            &crate::source::model::CfgPredicate::All(Vec::new()),
                            &signature.generics,
                            0,
                        ))
                {
                    self.spans.push(parameter.span());
                }
            }
        }
        syn::visit::visit_signature(self, signature);
        self.closure_parameters.truncate(count);
    }

    fn visit_field(&mut self, field: &'ast syn::Field) {
        let is_callback = field
            .ident
            .as_ref()
            .is_some_and(|name| name.unraw().to_string().starts_with("on_"));
        if !self.is_test && is_callback && self.is_closure_type(&field.ty) {
            self.spans.push(field.span());
        }
        syn::visit::visit_field(self, field);
    }
}
