use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::util::{
    build_shape_item_finding, find_kind_name, is_model_item, owns_implementation,
};
use crate::syntax::util::to_snake_case;
use syn::{Expr, ImplItem, Item, Stmt, Visibility};

pub(crate) struct ValueObjectShape;

impl ValueObjectShape {
    fn has_invariant(context: &RuleContext, ordinal: usize, identity: usize) -> bool {
        let index = &context.indexes[ordinal];
        is_model_item(context, ordinal, &index.items[identity]) && index.implementations.iter().filter(|implementation| implementation.syntax.trait_.is_none() && owns_implementation(index, identity, implementation)).any(|implementation| implementation.syntax.items.iter().any(|method| matches!(method, ImplItem::Fn(method) if method.sig.ident == "new" && method.block.stmts.iter().any(Self::is_assert))))
    }

    fn parser_returns(
        context: &RuleContext,
        ordinal: usize,
        parser: &crate::source::model::IndexedItem,
        identity: usize,
    ) -> bool {
        let Item::Fn(function) = &parser.syntax else {
            return false;
        };
        let syn::ReturnType::Type(_, typed) = &function.sig.output else {
            return false;
        };
        context
            .resolve_workspace_type(ordinal, parser.module, typed)
            .iter()
            .any(|target| target.index == ordinal && target.path.item == Some(identity))
    }

    fn is_assert(statement: &Stmt) -> bool {
        match statement {
            Stmt::Macro(statement) => statement.mac.path.is_ident("assert"),
            Stmt::Expr(Expr::Macro(expression), _) => expression.mac.path.is_ident("assert"),
            _ => false,
        }
    }

    fn is_construction(expression: &Expr, name: &str) -> bool {
        let path = match expression {
            Expr::Struct(expression) => Some(&expression.path),
            Expr::Call(call) => {
                if let Expr::Path(path) = call.func.as_ref() {
                    Some(&path.path)
                } else {
                    None
                }
            }
            Expr::Return(returned) => {
                return returned
                    .expr
                    .as_ref()
                    .is_some_and(|expression| Self::is_construction(expression, name));
            }
            _ => None,
        };
        path.is_some_and(|path| path.is_ident("Self") || path.is_ident(name))
    }

    fn invalid_constructor(function: &syn::ImplItemFn, name: &str) -> bool {
        function.block.stmts.iter().any(|statement| {
            if Self::is_assert(statement) {
                return false;
            }
            match statement {
                Stmt::Expr(expression, _) => !Self::is_construction(expression, name),
                _ => true,
            }
        })
    }

    fn check_value(context: &RuleContext, ordinal: usize, identity: usize) -> Option<RuleFinding> {
        let index = &context.indexes[ordinal];
        let item = &index.items[identity];
        let Item::Struct(structure) = &item.syntax else {
            return None;
        };
        if !is_model_item(context, ordinal, item) {
            return None;
        }
        let implementations: Vec<_> = index
            .implementations
            .iter()
            .filter(|implementation| {
                implementation.syntax.trait_.is_none()
                    && owns_implementation(index, identity, implementation)
            })
            .collect();
        let methods: Vec<_> = implementations
            .iter()
            .flat_map(|implementation| &implementation.syntax.items)
            .filter_map(|method| {
                if let ImplItem::Fn(method) = method {
                    Some(method)
                } else {
                    None
                }
            })
            .collect();
        let constructor = methods.iter().find(|method| {
            method.sig.ident == "new" && method.block.stmts.iter().any(Self::is_assert)
        })?;
        let exposed = structure
            .fields
            .iter()
            .any(|field| !matches!(field.vis, Visibility::Inherited));
        let missing_accessor = Self::missing_accessor(structure, constructor, &methods);
        (exposed || missing_accessor || Self::invalid_constructor(constructor, &item.name)).then(|| build_shape_item_finding(index, item, "ValueObjectShape", "a validated value object keeps fields private, asserts then constructs, and exposes a same-name accessor for each value"))
    }

    fn missing_accessor(
        structure: &syn::ItemStruct,
        constructor: &syn::ImplItemFn,
        methods: &[&syn::ImplItemFn],
    ) -> bool {
        let parameters = Self::tuple_names(constructor);
        structure
            .fields
            .iter()
            .enumerate()
            .any(|(position, field)| {
                let name = field
                    .ident
                    .as_ref()
                    .map(ToString::to_string)
                    .or_else(|| parameters.get(position).cloned().flatten());
                name.is_some_and(|name| {
                    !methods
                        .iter()
                        .any(|method| method.sig.ident == name && method.sig.receiver().is_some())
                })
            })
    }

    fn tuple_names(constructor: &syn::ImplItemFn) -> Vec<Option<String>> {
        let Some(Stmt::Expr(expression, _)) = constructor.block.stmts.last() else {
            return Vec::new();
        };
        let expression = match expression {
            Expr::Return(returned) => returned.expr.as_deref().unwrap_or(expression),
            _ => expression,
        };
        let Expr::Call(call) = expression else {
            return Vec::new();
        };
        call.args
            .iter()
            .map(|argument| match argument {
                Expr::Path(path) => path.path.get_ident().map(ToString::to_string),
                _ => None,
            })
            .collect()
    }
}

impl Rule for ValueObjectShape {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            findings.extend(
                (0..index.items.len())
                    .filter_map(|identity| Self::check_value(context, ordinal, identity)),
            );
            for parser in index.items.iter().filter(|item| {
                matches!(item.syntax, Item::Fn(_))
                    && item.name.starts_with("parse_")
                    && find_kind_name(context, ordinal, item.module) != "util"
            }) {
                let matching = index.items.iter().enumerate().any(|(identity, item)| {
                    parser.name == format!("parse_{}", to_snake_case(&item.name))
                        && Self::has_invariant(context, ordinal, identity)
                        && Self::parser_returns(context, ordinal, parser, identity)
                });
                if matching {
                    findings.push(build_shape_item_finding(
                        index,
                        parser,
                        "ValueObjectShape",
                        "a value-object parser lives in util",
                    ));
                }
            }
        }
        findings
    }
}
