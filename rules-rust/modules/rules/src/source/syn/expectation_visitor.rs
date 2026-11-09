use crate::source::model::Expectation;
use crate::source::model::ExpectationPosition;
use crate::source::model::ModuleId;
use crate::source::syn::ExpectationParser;
use crate::source::util::find_node_span;
use crate::source::util::parse_macro_arguments;
use proc_macro2::Span;
use syn::Arm;
use syn::AttrStyle;
use syn::Attribute;
use syn::Expr;
use syn::Field;
use syn::FnArg;
use syn::ForeignItem;
use syn::ImplItem;
use syn::Item;
use syn::Local;
use syn::Stmt;
use syn::TraitItem;
use syn::Variant;
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct ExpectationVisitor {
    file: String,
    module: ModuleId,
    expectations: Vec<Expectation>,
}

impl ExpectationVisitor {
    pub(crate) fn new(file: String, module: ModuleId) -> Self {
        Self {
            file,
            module,
            expectations: Vec::new(),
        }
    }

    fn collect(&mut self, attributes: &[Attribute], position: &ExpectationPosition) {
        for attribute in attributes {
            let line = u32::try_from(attribute.span().start().line).unwrap_or(u32::MAX);
            let parser = ExpectationParser::new(self.file.clone(), line, position.clone());
            if let Some(expectation) = parser.parse(attribute) {
                self.expectations.push(expectation);
            }
        }
    }

    fn collect_outer(&mut self, attributes: &[Attribute], span: Span) {
        let outer: Vec<Attribute> = attributes
            .iter()
            .filter(|attribute| matches!(attribute.style, AttrStyle::Outer))
            .cloned()
            .collect();
        self.collect(&outer, &ExpectationPosition::Node(find_node_span(span)));
    }

    fn find_expression_attributes(expression: &Expr) -> &[Attribute] {
        match expression {
            Expr::Array(inner) => &inner.attrs,
            Expr::Assign(inner) => &inner.attrs,
            Expr::Async(inner) => &inner.attrs,
            Expr::Await(inner) => &inner.attrs,
            Expr::Binary(inner) => &inner.attrs,
            Expr::Block(inner) => &inner.attrs,
            Expr::Break(inner) => &inner.attrs,
            Expr::Call(inner) => &inner.attrs,
            Expr::Cast(inner) => &inner.attrs,
            Expr::Closure(inner) => &inner.attrs,
            Expr::Const(inner) => &inner.attrs,
            Expr::Continue(inner) => &inner.attrs,
            Expr::Field(inner) => &inner.attrs,
            Expr::ForLoop(inner) => &inner.attrs,
            Expr::Group(inner) => &inner.attrs,
            Expr::If(inner) => &inner.attrs,
            Expr::Index(inner) => &inner.attrs,
            Expr::Infer(inner) => &inner.attrs,
            Expr::Let(inner) => &inner.attrs,
            Expr::Lit(inner) => &inner.attrs,
            Expr::Loop(inner) => &inner.attrs,
            Expr::Macro(inner) => &inner.attrs,
            Expr::Match(inner) => &inner.attrs,
            Expr::MethodCall(inner) => &inner.attrs,
            Expr::Paren(inner) => &inner.attrs,
            Expr::Path(inner) => &inner.attrs,
            Expr::Range(inner) => &inner.attrs,
            Expr::RawAddr(inner) => &inner.attrs,
            Expr::Reference(inner) => &inner.attrs,
            Expr::Repeat(inner) => &inner.attrs,
            Expr::Return(inner) => &inner.attrs,
            Expr::Struct(inner) => &inner.attrs,
            Expr::Try(inner) => &inner.attrs,
            Expr::TryBlock(inner) => &inner.attrs,
            Expr::Tuple(inner) => &inner.attrs,
            Expr::Unary(inner) => &inner.attrs,
            Expr::Unsafe(inner) => &inner.attrs,
            Expr::While(inner) => &inner.attrs,
            Expr::Yield(inner) => &inner.attrs,
            _ => &[],
        }
    }

    fn visit_module(&mut self, item: &syn::ItemMod) {
        let mut path = self.module.path.clone();
        path.push(item.ident.unraw().to_string());
        let module = ModuleId {
            crate_name: self.module.crate_name.clone(),
            target: self.module.target.clone(),
            path,
        };
        let outer: Vec<Attribute> = item
            .attrs
            .iter()
            .filter(|attribute| matches!(attribute.style, AttrStyle::Outer))
            .cloned()
            .collect();
        let inner: Vec<Attribute> = item
            .attrs
            .iter()
            .filter(|attribute| matches!(attribute.style, AttrStyle::Inner(_)))
            .cloned()
            .collect();
        let declaration = ExpectationPosition::ModuleDeclaration {
            module: module.clone(),
            declaration: find_node_span(item.span()),
        };
        self.collect(&outer, &declaration);
        self.collect(
            &inner,
            &ExpectationPosition::InlineModuleInner {
                module: module.clone(),
                declaration: find_node_span(item.span()),
            },
        );
        let Some((_, items)) = &item.content else {
            return;
        };
        let parent = std::mem::replace(&mut self.module, module);
        for child in items {
            self.visit_item(child);
        }
        self.module = parent;
    }

    pub(crate) fn collect_file(
        mut self,
        syntax: &syn::File,
        is_file_module: bool,
    ) -> Vec<Expectation> {
        let position = if is_file_module {
            ExpectationPosition::ModuleInner(self.module.clone())
        } else {
            ExpectationPosition::File
        };
        self.collect(&syntax.attrs, &position);
        for item in &syntax.items {
            self.visit_item(item);
        }
        self.expectations
    }
}

impl<'ast> Visit<'ast> for ExpectationVisitor {
    fn visit_expr(&mut self, expression: &'ast Expr) {
        self.collect_outer(
            Self::find_expression_attributes(expression),
            expression.span(),
        );
        syn::visit::visit_expr(self, expression);
    }

    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        for argument in &parse_macro_arguments(item) {
            self.visit_expr(argument);
        }
    }

    fn visit_item(&mut self, item: &'ast Item) {
        if let Item::Mod(module) = item {
            self.visit_module(module);
            return;
        }
        let attributes = match item {
            Item::Const(inner) => &inner.attrs,
            Item::Enum(inner) => &inner.attrs,
            Item::ExternCrate(inner) => &inner.attrs,
            Item::Fn(inner) => &inner.attrs,
            Item::ForeignMod(inner) => &inner.attrs,
            Item::Impl(inner) => &inner.attrs,
            Item::Macro(inner) => &inner.attrs,
            Item::Static(inner) => &inner.attrs,
            Item::Struct(inner) => &inner.attrs,
            Item::Trait(inner) => &inner.attrs,
            Item::TraitAlias(inner) => &inner.attrs,
            Item::Type(inner) => &inner.attrs,
            Item::Union(inner) => &inner.attrs,
            Item::Use(inner) => &inner.attrs,
            _ => return,
        };
        self.collect_outer(attributes, item.span());
        syn::visit::visit_item(self, item);
    }

    fn visit_impl_item(&mut self, item: &'ast ImplItem) {
        let attributes = match item {
            ImplItem::Const(inner) => &inner.attrs,
            ImplItem::Fn(inner) => &inner.attrs,
            ImplItem::Type(inner) => &inner.attrs,
            ImplItem::Macro(inner) => &inner.attrs,
            _ => return,
        };
        self.collect_outer(attributes, item.span());
        syn::visit::visit_impl_item(self, item);
    }

    fn visit_trait_item(&mut self, item: &'ast TraitItem) {
        let attributes = match item {
            TraitItem::Const(inner) => &inner.attrs,
            TraitItem::Fn(inner) => &inner.attrs,
            TraitItem::Type(inner) => &inner.attrs,
            TraitItem::Macro(inner) => &inner.attrs,
            _ => return,
        };
        self.collect_outer(attributes, item.span());
        syn::visit::visit_trait_item(self, item);
    }

    fn visit_foreign_item(&mut self, item: &'ast ForeignItem) {
        let attributes = match item {
            ForeignItem::Fn(inner) => &inner.attrs,
            ForeignItem::Static(inner) => &inner.attrs,
            ForeignItem::Type(inner) => &inner.attrs,
            ForeignItem::Macro(inner) => &inner.attrs,
            _ => return,
        };
        self.collect_outer(attributes, item.span());
        syn::visit::visit_foreign_item(self, item);
    }

    fn visit_field(&mut self, field: &'ast Field) {
        self.collect_outer(&field.attrs, field.span());
        syn::visit::visit_field(self, field);
    }

    fn visit_variant(&mut self, variant: &'ast Variant) {
        self.collect_outer(&variant.attrs, variant.span());
        syn::visit::visit_variant(self, variant);
    }

    fn visit_fn_arg(&mut self, argument: &'ast FnArg) {
        let attributes = match argument {
            FnArg::Receiver(receiver) => &receiver.attrs,
            FnArg::Typed(typed) => &typed.attrs,
        };
        self.collect_outer(attributes, argument.span());
        syn::visit::visit_fn_arg(self, argument);
    }

    fn visit_local(&mut self, local: &'ast Local) {
        self.collect_outer(&local.attrs, local.span());
        syn::visit::visit_local(self, local);
    }

    fn visit_stmt(&mut self, statement: &'ast Stmt) {
        if let Stmt::Macro(macro_statement) = statement {
            self.collect_outer(&macro_statement.attrs, statement.span());
        }
        syn::visit::visit_stmt(self, statement);
    }

    fn visit_arm(&mut self, arm: &'ast Arm) {
        self.collect_outer(&arm.attrs, arm.span());
        syn::visit::visit_arm(self, arm);
    }
}
