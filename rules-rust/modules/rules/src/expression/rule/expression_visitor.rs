use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::check::util::format_module_path;
use crate::source::model::SourceFile;
use crate::source::util::find_node_span;
use crate::source::util::parse_macro_arguments;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct ExpressionVisitor<'source> {
    file: &'source SourceFile,
    rule: &'static str,
    findings: Vec<RuleFinding>,
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
        let syn::Expr::Binary(binary) = condition else {
            return;
        };
        let is_boolean = matches!(binary.op, syn::BinOp::And(_) | syn::BinOp::Or(_));
        let is_multiline = condition.span().start().line != condition.span().end().line;
        if is_boolean && is_multiline && !Self::is_simple_operand(condition) {
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

    pub(crate) fn check_context(context: &RuleContext, rule: &'static str) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for file in context.crates.iter().flat_map(|sources| &sources.files) {
            let mut visitor = ExpressionVisitor {
                file,
                rule,
                findings: Vec::new(),
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
        if self.rule == "OperatorBeforeTrailingCall" {
            let parenthesized_arithmetic = matches!(&*expression.receiver,
                syn::Expr::Paren(paren) if matches!(&*paren.expr,
                    syn::Expr::Binary(binary) if Self::is_arithmetic(&binary.op)));
            let chained_arithmetic = Self::is_arithmetic_method(&expression.method)
                && matches!(&*expression.receiver, syn::Expr::MethodCall(receiver)
                    if Self::is_arithmetic_method(&receiver.method));
            if parenthesized_arithmetic || chained_arithmetic {
                self.report(expression.span(), "Use one arithmetic method before a trailing call, or name a multi-operator result first (clean-code.rs.md)");
            }
        }
        syn::visit::visit_expr_method_call(self, expression);
    }
}
