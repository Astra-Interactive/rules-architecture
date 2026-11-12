use crate::runtime::rule::FallibleStepCount;
use quote::ToTokens;
use syn::visit::Visit;

#[derive(Clone)]
pub(crate) struct FallibleStepVisitor {
    count: FallibleStepCount,
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
            count: FallibleStepCount::empty(),
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
                .filter_map(|branch| branch.break_targets[index].count.clone())
                .reduce(FallibleStepCount::merge);
            target.continued = branches
                .iter()
                .filter_map(|branch| branch.break_targets[index].continued.clone())
                .reduce(FallibleStepCount::merge);
        }
        self.maximum = branches
            .iter()
            .map(|branch| branch.maximum)
            .max()
            .unwrap_or(self.maximum);
        let continuation = branches
            .iter()
            .filter(|branch| branch.continues)
            .map(|branch| branch.count.clone())
            .reduce(FallibleStepCount::merge);
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

    fn parse_literal_value(expression: &syn::Expr) -> Option<i128> {
        match expression {
            syn::Expr::Lit(literal) => match &literal.lit {
                syn::Lit::Int(value) => value.base10_parse().ok(),
                _ => None,
            },
            syn::Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Neg(_)) => {
                Self::parse_literal_value(&unary.expr)?.checked_neg()
            }
            _ => None,
        }
    }

    fn resolve_pattern_range(pattern: &syn::Pat) -> Option<std::ops::RangeInclusive<i128>> {
        match Self::without_guard(pattern) {
            syn::Pat::Lit(literal) => {
                let value = Self::parse_literal_value(&syn::Expr::Lit(literal.clone()))?;
                Some(value..=value)
            }
            syn::Pat::Range(range) => {
                let start = match &range.start {
                    Some(start) => Self::parse_literal_value(start)?,
                    None => i128::MIN,
                };
                let end = match &range.end {
                    Some(end) => Self::parse_literal_value(end)?,
                    None => i128::MAX,
                };
                let has_exclusive_end =
                    range.end.is_some() && matches!(range.limits, syn::RangeLimits::HalfOpen(_));
                let end = if has_exclusive_end {
                    end.checked_sub(1)?
                } else {
                    end
                };
                Some(start..=end)
            }
            _ => None,
        }
    }

    fn covers_pattern(pattern: &syn::Pat, candidate: &syn::Pat) -> bool {
        let pattern = Self::without_guard(pattern);
        let candidate = Self::without_guard(candidate);
        matches!(pattern, syn::Pat::Wild(_))
            || pattern.to_token_stream().to_string() == candidate.to_token_stream().to_string()
            || Self::resolve_pattern_range(pattern)
                .zip(Self::resolve_pattern_range(candidate))
                .is_some_and(|(range, candidate)| {
                    range.start() <= candidate.start() && range.end() >= candidate.end()
                })
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

    fn visit_loop_iteration(
        &mut self,
        condition: Option<&syn::Expr>,
        body: &syn::Block,
    ) -> Option<FallibleStepCount> {
        let mut exit = None;
        if let Some(condition) = condition {
            self.visit_expr(condition);
            if self.continues && Self::known_boolean(condition) != Some(true) {
                exit = Some(self.count.clone());
            }
            if Self::known_boolean(condition) == Some(false) {
                self.continues = false;
            }
        }
        self.visit_block(body);
        exit
    }

    fn visit_loop_body(
        &mut self,
        label: Option<&syn::Label>,
        condition: Option<&syn::Expr>,
        body: &syn::Block,
        may_finish: bool,
    ) {
        if !self.continues {
            return;
        }
        let mut exits = may_finish
            .then(|| self.count.clone())
            .filter(|_| condition.is_none());
        self.break_targets.push(BreakTarget {
            label: label.map(|label| label.name.clone()),
            count: None,
            continued: None,
            is_loop: true,
        });
        loop {
            let entry = self.count.clone();
            if let Some(exit) = self.visit_loop_iteration(condition, body) {
                exits = Some(exits.map_or_else(|| exit.clone(), |count| count.merge(exit.clone())));
            }
            let Some(target) = self.break_targets.last_mut() else {
                break;
            };
            let mut next = target.continued.take();
            if self.continues {
                next = Some(next.map_or_else(
                    || self.count.clone(),
                    |count| count.merge(self.count.clone()),
                ));
            }
            let Some(next) = next else {
                break;
            };
            if may_finish && condition.is_none() {
                exits = Some(exits.map_or_else(|| next.clone(), |count| count.merge(next.clone())));
            }
            let joined = entry.clone().merge(next);
            if joined == entry {
                break;
            }
            self.count = joined;
            self.continues = true;
        }
        if let Some(count) = self.break_targets.pop().and_then(|target| target.count) {
            exits = Some(exits.map_or_else(|| count.clone(), |exit| exit.merge(count.clone())));
        }
        self.continues = exits.is_some();
        if let Some(count) = exits {
            self.count = count;
        }
    }
}

impl<'ast> Visit<'ast> for FallibleStepVisitor {
    fn visit_expr(&mut self, expression: &'ast syn::Expr) {
        if self.continues {
            syn::visit::visit_expr(self, expression);
        }
    }

    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        let arguments = crate::source::util::parse_macro_arguments(item);
        for argument in &arguments {
            self.visit_expr(argument);
        }
    }

    fn visit_expr_try(&mut self, expression: &'ast syn::ExprTry) {
        self.visit_expr(&expression.expr);
        if self.continues {
            let site = std::ptr::from_ref(expression) as usize;
            self.count.add(site);
            self.maximum = self.maximum.max(self.count.compute_maximum());
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
            label: Some(label.name.clone()),
            count: None,
            continued: None,
            is_loop: false,
        });
        self.visit_block(&expression.block);
        if let Some(target) = self.break_targets.pop()
            && let Some(count) = target.count
        {
            self.count = if self.continues {
                self.count.clone().merge(count)
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
        let target = self
            .break_targets
            .iter_mut()
            .rev()
            .find(|target| match &expression.label {
                Some(label) => target
                    .label
                    .as_ref()
                    .is_some_and(|target| target.ident == label.ident),
                None => target.is_loop,
            });
        if let Some(target) = target {
            target.count = Some(target.count.clone().map_or_else(
                || self.count.clone(),
                |count| count.merge(self.count.clone()),
            ));
        }
        self.continues = false;
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let Some(init) = &local.init {
            let mut initializer = self.clone();
            for target in &mut initializer.break_targets {
                target.count = None;
                target.continued = None;
            }
            initializer.visit_expr(&init.expr);
            self.continues = initializer.continues;
            for (target, initialized) in
                self.break_targets.iter_mut().zip(initializer.break_targets)
            {
                if initialized.count.is_some() {
                    target.count = Some(target.count.clone().map_or_else(
                        || self.count.clone(),
                        |count| count.merge(self.count.clone()),
                    ));
                }
                if initialized.continued.is_some() {
                    target.continued = Some(target.continued.clone().map_or_else(
                        || self.count.clone(),
                        |count| count.merge(self.count.clone()),
                    ));
                }
            }
        }
    }

    fn visit_item(&mut self, _: &'ast syn::Item) {}

    fn visit_expr_closure(&mut self, _: &'ast syn::ExprClosure) {}

    fn visit_expr_async(&mut self, _: &'ast syn::ExprAsync) {}

    fn visit_expr_loop(&mut self, expression: &'ast syn::ExprLoop) {
        self.visit_loop_body(expression.label.as_ref(), None, &expression.body, false);
    }

    fn visit_expr_for_loop(&mut self, expression: &'ast syn::ExprForLoop) {
        self.visit_expr(&expression.expr);
        self.visit_loop_body(expression.label.as_ref(), None, &expression.body, true);
    }

    fn visit_expr_while(&mut self, expression: &'ast syn::ExprWhile) {
        self.visit_loop_body(
            expression.label.as_ref(),
            Some(&expression.cond),
            &expression.body,
            true,
        );
    }

    fn visit_expr_continue(&mut self, expression: &'ast syn::ExprContinue) {
        let label = expression.label.as_ref();
        let target = self.break_targets.iter_mut().rev().find(|target| {
            let is_label_match = label.is_none_or(|label| {
                target
                    .label
                    .as_ref()
                    .is_some_and(|target| target.ident == label.ident)
            });
            target.is_loop && is_label_match
        });
        if let Some(target) = target {
            target.continued = Some(target.continued.clone().map_or_else(
                || self.count.clone(),
                |count| count.merge(self.count.clone()),
            ));
        }
        self.continues = false;
    }

    fn visit_expr_return(&mut self, expression: &'ast syn::ExprReturn) {
        if let Some(value) = &expression.expr {
            self.visit_expr(value);
        }
        self.continues = false;
    }
}

#[derive(Clone)]
struct BreakTarget {
    label: Option<syn::Lifetime>,
    count: Option<FallibleStepCount>,
    continued: Option<FallibleStepCount>,
    is_loop: bool,
}
