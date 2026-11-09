use quote::ToTokens;
use syn::visit::Visit;

#[derive(Clone)]
pub(crate) struct FallibleStepVisitor {
    count: usize,
    maximum: usize,
    continues: bool,
    break_targets: Vec<BreakTarget>,
}

impl FallibleStepVisitor {
    fn known_boolean(expression: &syn::Expr) -> Option<bool> {
        match expression {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Bool(value),
                ..
            }) => Some(value.value),
            syn::Expr::Paren(paren) => Self::known_boolean(&paren.expr),
            syn::Expr::Group(group) => Self::known_boolean(&group.expr),
            syn::Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Not(_)) => {
                Self::known_boolean(&unary.expr).map(|value| !value)
            }
            syn::Expr::Binary(binary) => {
                let left = Self::known_boolean(&binary.left);
                let right = Self::known_boolean(&binary.right);
                match (&binary.op, left, right) {
                    (syn::BinOp::And(_), Some(false), _) | (syn::BinOp::And(_), _, Some(false)) => {
                        Some(false)
                    }
                    (syn::BinOp::Or(_), Some(true), _) | (syn::BinOp::Or(_), _, Some(true)) => {
                        Some(true)
                    }
                    (syn::BinOp::And(_) | syn::BinOp::Or(_), Some(value), Some(_)) => Some(value),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub(crate) fn count(expression: &syn::Expr) -> usize {
        let mut visitor = Self {
            count: 0,
            maximum: 0,
            continues: true,
            break_targets: Vec::new(),
        };
        visitor.visit_expr(expression);
        visitor.maximum
    }

    fn merge(&mut self, branches: &[Self]) {
        for (index, target) in self.break_targets.iter_mut().enumerate() {
            target.count = branches
                .iter()
                .filter_map(|branch| branch.break_targets[index].count)
                .max();
        }
        self.maximum = branches
            .iter()
            .map(|branch| branch.maximum)
            .max()
            .unwrap_or(self.maximum);
        let continuation = branches
            .iter()
            .filter(|branch| branch.continues)
            .map(|branch| branch.count)
            .max();
        self.continues = continuation.is_some();
        if let Some(count) = continuation {
            self.count = count;
        }
    }

    fn without_guard(pattern: &syn::Pat) -> &syn::Pat {
        match pattern {
            syn::Pat::Guard(guard) => Self::without_guard(&guard.pat),
            syn::Pat::Paren(paren) => Self::without_guard(&paren.pat),
            _ => pattern,
        }
    }

    fn covers_pattern(pattern: &syn::Pat, candidate: &syn::Pat) -> bool {
        let pattern = Self::without_guard(pattern);
        let candidate = Self::without_guard(candidate);
        matches!(pattern, syn::Pat::Wild(_))
            || pattern.to_token_stream().to_string() == candidate.to_token_stream().to_string()
    }

    fn visit_failed_guard(&mut self, pattern: &syn::Pat) {
        if let syn::Pat::Guard(guard) = pattern {
            self.visit_expr(&guard.guard);
            if Self::known_boolean(&guard.guard) == Some(true) {
                self.continues = false;
            }
        } else {
            self.continues = false;
        }
    }

    fn visit_match_prefix(&self, previous: &[syn::Arm], candidate: &syn::Pat) -> Self {
        let mut branch = self.clone();
        for earlier in previous {
            if Self::covers_pattern(&earlier.pat, candidate) {
                branch.visit_failed_guard(&earlier.pat);
            }
        }
        branch
    }

    fn visit_match_arm(&self, previous: &[syn::Arm], arm: &syn::Arm) -> Self {
        let mut candidates = vec![self.visit_match_prefix(previous, &arm.pat)];
        for earlier in previous {
            if Self::covers_pattern(&arm.pat, &earlier.pat) {
                candidates.push(self.visit_match_prefix(previous, &earlier.pat));
            }
        }
        let mut branch = self.clone();
        branch.merge(&candidates);
        if let syn::Pat::Guard(guard) = &arm.pat {
            branch.visit_expr(&guard.guard);
            if Self::known_boolean(&guard.guard) == Some(false) {
                branch.continues = false;
            }
        }
        branch.visit_expr(&arm.body);
        branch
    }
}

impl<'ast> Visit<'ast> for FallibleStepVisitor {
    fn visit_expr(&mut self, expression: &'ast syn::Expr) {
        if self.continues {
            syn::visit::visit_expr(self, expression);
        }
    }

    fn visit_expr_try(&mut self, expression: &'ast syn::ExprTry) {
        self.visit_expr(&expression.expr);
        if self.continues {
            self.count = (self.count + 1).min(2);
            self.maximum = self.maximum.max(self.count);
        }
    }

    fn visit_expr_binary(&mut self, expression: &'ast syn::ExprBinary) {
        self.visit_expr(&expression.left);
        match (&expression.op, Self::known_boolean(&expression.left)) {
            (syn::BinOp::And(_), Some(false)) | (syn::BinOp::Or(_), Some(true)) => {}
            (syn::BinOp::And(_) | syn::BinOp::Or(_), None) => {
                let mut evaluated = self.clone();
                evaluated.visit_expr(&expression.right);
                self.merge(&[self.clone(), evaluated]);
            }
            _ => self.visit_expr(&expression.right),
        }
    }

    fn visit_expr_if(&mut self, expression: &'ast syn::ExprIf) {
        self.visit_expr(&expression.cond);
        let mut then_branch = self.clone();
        then_branch.visit_block(&expression.then_branch);
        let mut else_branch = self.clone();
        if let Some((_, branch)) = &expression.else_branch {
            else_branch.visit_expr(branch);
        }
        match Self::known_boolean(&expression.cond) {
            Some(true) => *self = then_branch,
            Some(false) => *self = else_branch,
            None => self.merge(&[then_branch, else_branch]),
        }
    }

    fn visit_expr_match(&mut self, expression: &'ast syn::ExprMatch) {
        self.visit_expr(&expression.expr);
        let mut branches = Vec::new();
        for (index, arm) in expression.arms.iter().enumerate() {
            let previous = &expression.arms[..index];
            branches.push(self.visit_match_arm(previous, arm));
        }
        self.merge(&branches);
    }

    fn visit_expr_block(&mut self, expression: &'ast syn::ExprBlock) {
        let Some(label) = &expression.label else {
            self.visit_block(&expression.block);
            return;
        };
        self.break_targets.push(BreakTarget {
            label: label.name.clone(),
            count: None,
        });
        self.visit_block(&expression.block);
        if let Some(target) = self.break_targets.pop()
            && let Some(count) = target.count
        {
            self.count = if self.continues {
                self.count.max(count)
            } else {
                count
            };
            self.continues = true;
        }
    }

    fn visit_expr_break(&mut self, expression: &'ast syn::ExprBreak) {
        if let Some(value) = &expression.expr {
            self.visit_expr(value);
        }
        if !self.continues {
            return;
        }
        if let Some(label) = &expression.label
            && let Some(target) = self
                .break_targets
                .iter_mut()
                .rev()
                .find(|target| target.label.ident == label.ident)
        {
            target.count = Some(
                target
                    .count
                    .map_or(self.count, |count| count.max(self.count)),
            );
        }
        self.continues = false;
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let Some(init) = &local.init {
            let mut initializer = self.clone();
            for target in &mut initializer.break_targets {
                target.count = None;
            }
            initializer.visit_expr(&init.expr);
            self.continues = initializer.continues;
            for (target, initialized) in
                self.break_targets.iter_mut().zip(initializer.break_targets)
            {
                if initialized.count.is_some() {
                    target.count = Some(
                        target
                            .count
                            .map_or(self.count, |count| count.max(self.count)),
                    );
                }
            }
        }
    }

    fn visit_item(&mut self, _: &'ast syn::Item) {}

    fn visit_expr_closure(&mut self, _: &'ast syn::ExprClosure) {}

    fn visit_expr_async(&mut self, _: &'ast syn::ExprAsync) {}

    fn visit_expr_loop(&mut self, _: &'ast syn::ExprLoop) {}

    fn visit_expr_for_loop(&mut self, _: &'ast syn::ExprForLoop) {}

    fn visit_expr_while(&mut self, _: &'ast syn::ExprWhile) {}

    fn visit_expr_return(&mut self, expression: &'ast syn::ExprReturn) {
        if let Some(value) = &expression.expr {
            self.visit_expr(value);
        }
        self.continues = false;
    }
}

#[derive(Clone)]
struct BreakTarget {
    label: syn::Lifetime,
    count: Option<usize>,
}
