#[derive(Clone, Copy)]
pub(crate) enum DocTarget<'ast> {
    NoSubject,
    Item(&'ast syn::Item),
    Member(&'ast syn::Ident),
}
