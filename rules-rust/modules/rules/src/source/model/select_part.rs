pub(crate) enum SelectPart {
    Pattern(syn::Pat),
    Expression(syn::Expr),
    BranchEnd,
}
