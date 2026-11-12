pub(crate) enum MacroBody {
    Expressions(Vec<syn::Expr>),
    Items(Vec<syn::Item>),
    Statements(Vec<syn::Stmt>),
    Opaque,
}
