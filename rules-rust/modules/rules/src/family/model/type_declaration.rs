pub(crate) struct TypeDeclaration<'syntax> {
    pub(crate) item: &'syntax syn::Item,
    pub(crate) name: &'syntax syn::Ident,
}
