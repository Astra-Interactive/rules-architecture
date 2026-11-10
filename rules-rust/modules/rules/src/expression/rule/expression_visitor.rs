use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::check::util::format_module_path;
use crate::source::model::SourceFile;
use crate::source::util::find_node_span;
use crate::source::util::parse_macro_arguments;
use std::collections::HashSet;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct ExpressionVisitor<'source> {
    file: &'source SourceFile,
    rule: &'static str,
    findings: Vec<RuleFinding>,
    operator_traits: HashSet<&'static str>,
    trailing_call: bool,
}

impl ExpressionVisitor<'_> {
    fn report(&mut self, span: proc_macro2::Span, message: &str) {
        self.findings.push(build_node_finding(
            self.rule,
            &self.file.path,
            find_node_span(span),
            format_module_path(&self.file.module.crate_name, &self.file.module.path),
            message.to_owned(),
        ));
    }

    fn is_simple_operand(expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Path(_) | syn::Expr::Lit(_) => true,
            syn::Expr::Unary(unary) => {
                matches!(unary.op, syn::UnOp::Not(_)) && matches!(&*unary.expr, syn::Expr::Path(_))
            }
            syn::Expr::Binary(binary)
                if matches!(binary.op, syn::BinOp::And(_) | syn::BinOp::Or(_)) =>
            {
                Self::is_simple_operand(&binary.left) && Self::is_simple_operand(&binary.right)
            }
            _ => false,
        }
    }

    fn check_condition(&mut self, condition: &syn::Expr) {
        if self.rule != "BooleanOperatorWrapping" {
            return;
        }
        let is_multiline = condition.span().start().line != condition.span().end().line;
        if is_multiline && !Self::is_simple_operand(condition) {
            self.report(condition.span(), "Split complex multiline boolean clauses into guards and named values (clean-code.rs.md)");
        }
    }

    fn check_guard(&mut self, block: &syn::Block) {
        if self.rule != "ClosureSingleIfGuard" || block.stmts.len() != 1 {
            return;
        }
        let syn::Stmt::Expr(syn::Expr::If(condition), _) = &block.stmts[0] else {
            return;
        };
        if condition.else_branch.is_none() && condition.then_branch.stmts.len() >= 2 {
            self.report(condition.span(), "Use an early-return guard for a closure or async body with multiple conditional statements (clean-code.rs.md)");
        }
    }

    fn is_arithmetic(operator: &syn::BinOp) -> bool {
        matches!(
            operator,
            syn::BinOp::Add(_)
                | syn::BinOp::Sub(_)
                | syn::BinOp::Mul(_)
                | syn::BinOp::Div(_)
                | syn::BinOp::Rem(_)
        )
    }

    fn is_arithmetic_method(name: &syn::Ident) -> bool {
        ["add", "sub", "mul", "div", "rem"]
            .iter()
            .any(|word| name == word)
    }

    fn is_method_call(expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::MethodCall(_) => true,
            syn::Expr::Paren(paren) => Self::is_method_call(&paren.expr),
            syn::Expr::Group(group) => Self::is_method_call(&group.expr),
            syn::Expr::Try(attempt) => Self::is_method_call(&attempt.expr),
            syn::Expr::Await(awaited) => Self::is_method_call(&awaited.base),
            _ => false,
        }
    }

    fn find_operator_trait(name: &syn::Ident) -> Option<&'static str> {
        ["Add", "Sub", "Mul", "Div", "Rem"]
            .into_iter()
            .find(|word| name == word)
    }

    fn find_method_trait(method: &syn::Ident) -> Option<&'static str> {
        match method.to_string().as_str() {
            "add" => Some("Add"),
            "sub" => Some("Sub"),
            "mul" => Some("Mul"),
            "div" => Some("Div"),
            "rem" => Some("Rem"),
            _ => None,
        }
    }

    fn is_operator_module(prefix: &[String]) -> bool {
        let [root, module] = prefix else {
            return false;
        };
        (root == "std" || root == "core") && module == "ops"
    }

    fn collect_operator_traits(
        tree: &syn::UseTree,
        prefix: &[String],
        traits: &mut HashSet<&'static str>,
    ) {
        let in_ops = Self::is_operator_module(prefix);
        match tree {
            syn::UseTree::Path(path) => {
                let mut prefix = prefix.to_vec();
                prefix.push(path.ident.to_string());
                Self::collect_operator_traits(&path.tree, &prefix, traits);
            }
            syn::UseTree::Group(group) => {
                for tree in &group.items {
                    Self::collect_operator_traits(tree, prefix, traits);
                }
            }
            syn::UseTree::Name(name) if in_ops => {
                traits.extend(Self::find_operator_trait(&name.ident));
            }
            syn::UseTree::Rename(rename) if in_ops => {
                traits.extend(Self::find_operator_trait(&rename.ident));
            }
            syn::UseTree::Glob(_) if in_ops => traits.extend(["Add", "Sub", "Mul", "Div", "Rem"]),
            _ => {}
        }
    }

    fn collect_operator_imports(items: &[syn::Item]) -> HashSet<&'static str> {
        let mut traits = HashSet::new();
        for item in items {
            match item {
                syn::Item::Use(import) => {
                    Self::collect_operator_traits(&import.tree, &[], &mut traits);
                }
                syn::Item::Mod(module) => {
                    if let Some((_, children)) = &module.content {
                        traits.extend(Self::collect_operator_imports(children));
                    }
                }
                _ => {}
            }
        }
        traits
    }

    fn is_imported_operator_call(&self, call: &syn::ExprMethodCall) -> bool {
        call.args.len() == 1
            && Self::find_method_trait(&call.method)
                .is_some_and(|name| self.operator_traits.contains(name))
    }

    pub(crate) fn check_context(context: &RuleContext, rule: &'static str) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for file in context.crates.iter().flat_map(|sources| &sources.files) {
            let mut visitor = ExpressionVisitor {
                file,
                rule,
                findings: Vec::new(),
                operator_traits: Self::collect_operator_imports(&file.syntax.items),
                trailing_call: false,
            };
            visitor.visit_file(&file.syntax);
            findings.extend(visitor.findings);
        }
        findings
    }
}

impl<'ast> Visit<'ast> for ExpressionVisitor<'_> {
    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        for argument in &parse_macro_arguments(item) {
            self.visit_expr(argument);
        }
    }

    fn visit_expr_if(&mut self, expression: &'ast syn::ExprIf) {
        self.check_condition(&expression.cond);
        syn::visit::visit_expr_if(self, expression);
    }

    fn visit_expr_while(&mut self, expression: &'ast syn::ExprWhile) {
        self.check_condition(&expression.cond);
        syn::visit::visit_expr_while(self, expression);
    }

    fn visit_pat_guard(&mut self, pattern: &'ast syn::PatGuard) {
        self.check_condition(&pattern.guard);
        syn::visit::visit_pat_guard(self, pattern);
    }

    fn visit_expr_closure(&mut self, expression: &'ast syn::ExprClosure) {
        match &*expression.body {
            syn::Expr::Block(block) => self.check_guard(&block.block),
            syn::Expr::If(condition)
                if self.rule == "ClosureSingleIfGuard"
                    && condition.else_branch.is_none()
                    && condition.then_branch.stmts.len() >= 2 =>
            {
                self.report(
                    condition.span(),
                    "Use an early-return guard (clean-code.rs.md)",
                );
            }
            _ => {}
        }
        syn::visit::visit_expr_closure(self, expression);
    }

    fn visit_expr_async(&mut self, expression: &'ast syn::ExprAsync) {
        self.check_guard(&expression.block);
        syn::visit::visit_expr_async(self, expression);
    }

    fn visit_expr_method_call(&mut self, expression: &'ast syn::ExprMethodCall) {
        let has_trailing_call = std::mem::take(&mut self.trailing_call);
        if self.rule == "OperatorBeforeTrailingCall" {
            let parenthesized_arithmetic = matches!(&*expression.receiver,
                syn::Expr::Paren(paren) if matches!(&*paren.expr,
                    syn::Expr::Binary(binary) if Self::is_arithmetic(&binary.op)));
            let chained_arithmetic = Self::is_arithmetic_method(&expression.method)
                && matches!(&*expression.receiver, syn::Expr::MethodCall(receiver)
                    if Self::is_arithmetic_method(&receiver.method));
            let lone_call = !has_trailing_call && self.is_imported_operator_call(expression);
            if parenthesized_arithmetic || chained_arithmetic || lone_call {
                self.report(expression.span(), "Use one arithmetic method before a trailing call, or name a multi-operator result first (clean-code.rs.md)");
            }
        }
        self.trailing_call = Self::is_method_call(&expression.receiver);
        self.visit_expr(&expression.receiver);
        self.trailing_call = false;
        for argument in &expression.args {
            self.visit_expr(argument);
        }
    }
}
