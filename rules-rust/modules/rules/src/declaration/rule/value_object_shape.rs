use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::rule::InvariantVisitor;
use crate::module::util::{
    build_shape_item_finding, find_kind_name, is_model_item, owns_implementation,
};
use crate::syntax::util::is_accessor_adapter;
use crate::syntax::util::to_snake_case;
use syn::{Expr, ImplItem, Item, Stmt, Visibility};

pub(crate) struct ValueObjectShape;

impl ValueObjectShape {
    fn accesses_field(expression: &Expr, member: &syn::Member) -> bool {
        match expression {
            Expr::Field(field) => {
                field.member == *member
                    && matches!(field.base.as_ref(), Expr::Path(path) if path.path.is_ident("self"))
            }
            Expr::Reference(reference) => Self::accesses_field(&reference.expr, member),
            Expr::MethodCall(call) if is_accessor_adapter(call) => {
                Self::accesses_field(&call.receiver, member)
            }
            Expr::Return(returned) => returned
                .expr
                .as_deref()
                .is_some_and(|expression| Self::accesses_field(expression, member)),
            Expr::Paren(paren) => Self::accesses_field(&paren.expr, member),
            _ => false,
        }
    }

    fn is_enum_error_result(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        typed: &syn::Type,
    ) -> bool {
        let syn::Type::Path(path) = typed else {
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
        context
            .resolve_workspace_type(ordinal, module, error)
            .iter()
            .any(|target| {
                target.path.item.is_some_and(|item| {
                    matches!(
                        context.indexes[target.index].items[item].syntax,
                        Item::Enum(_)
                    )
                })
            })
    }

    fn has_enum_error(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        typed: &syn::Type,
    ) -> bool {
        context
            .resolve_type_syntax(ordinal, module, typed)
            .iter()
            .any(|resolved| {
                Self::is_enum_error_result(
                    context,
                    resolved.index,
                    resolved.module,
                    &resolved.typed,
                )
            })
    }

    fn parser_is_invalid(
        context: &RuleContext,
        ordinal: usize,
        parser: &crate::source::model::IndexedItem,
        identity: usize,
    ) -> bool {
        use syn::visit::Visit;
        let Item::Fn(function) = &parser.syntax else {
            return false;
        };
        let syn::ReturnType::Type(_, typed) = &function.sig.output else {
            return true;
        };
        let result = context
            .resolve_outer_external_type_paths(ordinal, parser.module, typed)
            .iter()
            .any(|path| {
                ["std", "core"].contains(&path.package.as_str())
                    && path.segments.last().is_some_and(|name| name == "Result")
            });
        let mut visitor = ParserConstructionVisitor {
            context,
            ordinal,
            module: parser.module,
            identity,
            constructed: false,
            checked: false,
            invalid: false,
        };
        visitor.visit_block(&function.block);
        !result
            || !Self::parser_returns(context, ordinal, parser, identity)
            || !Self::has_enum_error(context, ordinal, parser.module, typed)
            || visitor.invalid
            || (visitor.constructed && !visitor.checked)
    }

    fn has_invariant(context: &RuleContext, ordinal: usize, identity: usize) -> bool {
        let index = &context.indexes[ordinal];
        is_model_item(context, ordinal, &index.items[identity]) && index.implementations.iter().filter(|implementation| implementation.syntax.trait_.is_none() && owns_implementation(index, identity, implementation)).any(|implementation| implementation.syntax.items.iter().any(|method| matches!(method, ImplItem::Fn(method) if method.sig.ident == "new" && InvariantVisitor::calls_assertion(&method.block))))
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
            Stmt::Macro(statement) => InvariantVisitor::is_assertion(&statement.mac.path),
            Stmt::Expr(Expr::Macro(expression), _) => {
                InvariantVisitor::is_assertion(&expression.mac.path)
            }
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
        function
            .block
            .stmts
            .iter()
            .enumerate()
            .any(|(position, statement)| {
                if Self::is_assert(statement) {
                    return false;
                }
                match statement {
                    Stmt::Expr(expression, _) => {
                        position + 1 != function.block.stmts.len()
                            || !Self::is_construction(expression, name)
                    }
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
            method.sig.ident == "new" && InvariantVisitor::calls_assertion(&method.block)
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
                let member = field.ident.as_ref().map_or_else(|| syn::Member::Unnamed(syn::Index::from(position)), |name| syn::Member::Named(name.clone()));
                let is_accessor = |method: &&&syn::ImplItemFn| method.sig.receiver().is_some() && matches!(method.block.stmts.as_slice(), [Stmt::Expr(expression, _)] if Self::accesses_field(expression, &member));
                match name {
                    Some(name) => !methods
                        .iter()
                        .any(|method| method.sig.ident == name && is_accessor(&method)),
                    None => !methods.iter().any(|method| is_accessor(&method)),
                }
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
                matches!(item.syntax, Item::Fn(_)) && item.name.starts_with("parse_")
            }) {
                let matching = index.items.iter().enumerate().any(|(identity, item)| {
                    parser.name == format!("parse_{}", to_snake_case(&item.name))
                        && Self::has_invariant(context, ordinal, identity)
                        && (find_kind_name(context, ordinal, parser.module) != "util"
                            || Self::parser_is_invalid(context, ordinal, parser, identity))
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

struct ParserConstructionVisitor<'context> {
    context: &'context RuleContext,
    ordinal: usize,
    module: usize,
    identity: usize,
    constructed: bool,
    checked: bool,
    invalid: bool,
}

impl<'ast> syn::visit::Visit<'ast> for ParserConstructionVisitor<'_> {
    fn visit_expr_if(&mut self, branch: &'ast syn::ExprIf) {
        let never = matches!(branch.cond.as_ref(), Expr::Lit(value) if matches!(&value.lit, syn::Lit::Bool(value) if !value.value));
        if never {
            if let Some((_, expression)) = &branch.else_branch {
                self.visit_expr(expression);
            }
            return;
        }
        syn::visit::visit_expr_if(self, branch);
    }

    fn visit_expr_return(&mut self, returned: &'ast syn::ExprReturn) {
        self.checked |= matches!(returned.expr.as_deref(), Some(Expr::Call(call)) if matches!(call.func.as_ref(), Expr::Path(path) if path.path.is_ident("Err")));
        syn::visit::visit_expr_return(self, returned);
    }

    fn visit_expr_try(&mut self, attempt: &'ast syn::ExprTry) {
        self.checked |= matches!(attempt.expr.as_ref(), Expr::Call(_) | Expr::MethodCall(_));
        syn::visit::visit_expr_try(self, attempt);
    }

    fn visit_expr_match(&mut self, matched: &'ast syn::ExprMatch) {
        self.checked |= matched
            .arms
            .iter()
            .any(|arm| rejects_with_err(arm.body.as_ref()));
        syn::visit::visit_expr_match(self, matched);
    }

    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if let Expr::Path(path) = call.func.as_ref() {
            let mut path = path.path.clone();
            if path
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "new")
            {
                path.segments.pop();
                path.segments.pop_punct();
                let constructs = self
                    .context
                    .resolve_workspace_path(self.ordinal, self.module, &path)
                    .iter()
                    .any(|target| {
                        target.index == self.ordinal && target.path.item == Some(self.identity)
                    });
                self.invalid |= constructs && !self.checked;
                self.constructed |= constructs;
            }
        }
        syn::visit::visit_expr_call(self, call);
    }

    fn visit_expr_closure(&mut self, _: &'ast syn::ExprClosure) {}
    fn visit_expr_async(&mut self, _: &'ast syn::ExprAsync) {}
}

fn rejects_with_err(expression: &Expr) -> bool {
    let rejection = match expression {
        Expr::Return(returned) => returned.expr.as_deref(),
        other => Some(other),
    };
    matches!(rejection, Some(Expr::Call(call)) if matches!(call.func.as_ref(), Expr::Path(path) if path.path.is_ident("Err")))
}
